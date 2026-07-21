//! Offline, external scope attestations for immutable learned-agent opinions.

use std::{collections::BTreeSet, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    core::stable_hash_string,
    data::{
        DataSnapshot, historical_replay_dataset_digest_v0, snapshot_id_from_semantic_digest_v1,
    },
};

use super::cycle_risk_shadow::MOMENTUM_AGENT_ID_V0;
use super::{
    AgentOpinionRelationshipV0, BtcHistoricalRegimeConfigV0, BtcHistoricalRegimeV0,
    BtcTemporalRegimeRefV0, CYCLE_RISK_SHADOW_AGENT_ID_V0, CycleRiskOpinionAdapterContextV0,
    CycleRiskProspectiveChallengeStatusV0, CycleRiskProspectiveLocalStateV0,
    CycleRiskProspectiveTournamentCapsuleV0, CycleRiskShadowConfigV0, EvidenceUsageClassV0,
    HistoricalEvidencePolicyV0, LearnedAgentObjectiveV0, MomentumCandleV0,
    MomentumLearningCampaignConfigV0, ProspectiveChallengeLocalStateV0,
    ProspectiveChallengeStatusV0, ProspectiveEvidenceRowRefV0, ProspectiveLabelStatusV0,
    ProspectivePredictionEventV0, ProspectiveShadowOutcomeV0, TemporalRegimeSegmentationPolicyV0,
    append_cycle_risk_external_row_and_event_v0, append_prospective_prediction_event_v0,
    append_prospective_vault_row_v0, append_shadow_deliberation_v0,
    assess_momentum_campaign_sufficiency_v0, build_momentum_features_v0,
    build_momentum_learning_windows_v0, build_momentum_sequence_examples_v0,
    close_btc_temporal_regime_result_v0, freeze_btc_historical_regime_packs_v0,
    frozen_mamba3_encoder_from_seed_v0, new_shadow_deliberation_ledger_v0,
    reconstruct_cycle_risk_opinion_from_regime_v0, replay_btc_shadow_deliberations_v0,
    run_btc_historical_regime_campaigns_v0, run_cycle_risk_shadow_regime_v0,
    segment_btc_historical_regimes_v0, validate_cycle_risk_prospective_capsule_v0,
    validate_cycle_risk_prospective_local_state_v0, validate_prospective_challenge_local_state_v0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalObservationScopeKindV0 {
    HistoricalRegimePack,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanonicalScopeLineageStatusV0 {
    SameSnapshot,
    CertifiedEquivalentRows,
    DifferentRows,
    DifferentSeries,
    DifferentProvider,
    DifferentInformationCutoff,
    InsufficientEvidence,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpinionScopeAttestationStatusV0 {
    Verified,
    MissingCanonicalScope,
    SourceDigestMismatch,
    AgentMismatch,
    ObjectiveMismatch,
    AmbiguousSource,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RawObservationScopeAlignmentV0 {
    ExactSameScope,
    CertifiedEquivalentRows,
    SameLineagePartialOverlap,
    DifferentInformationCutoff,
    Disjoint,
    Ambiguous,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectiveSampleAlignmentV0 {
    ExactSameAnchorSets,
    SameRawScopeDifferentAnchorSets,
    PartiallyOverlappingAnchorSets,
    DisjointAnchorSets,
    AnchorIdentityUnavailable,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectiveHorizonAlignmentV0 {
    SameHorizonDifferentObjective,
    DifferentHorizonCompatibleForRegimeSummary,
    DifferentHorizonNotComparable,
    MissingHorizonIdentity,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LearnedAgentScopeComparabilityV0 {
    ExactDecisionScopeComparable,
    RegimeSummaryComparableWithCaveats,
    SharedRawEvidenceButDifferentEffectiveSamples,
    PartialOverlapOnly,
    DifferentInformationCutoff,
    DisjointEvidence,
    AmbiguousMapping,
    Incomparable,
    TechnicalFailure,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeMappingRegistryStatusV0 {
    FullyMapped,
    FullyMappedWithCaveats,
    PartiallyMapped,
    Ambiguous,
    Empty,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregateDeliberationCompositionStatusV0 {
    FullyComposed,
    FullyComposedWithScopeCaveats,
    PartiallyMappedNotComposable,
    AmbiguousNotComposable,
    EmptyNotComposable,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregateRelationshipSummaryV0 {
    AllScopesBothAbstained,
    MostlyAbstained,
    TensionObserved,
    OrthogonalAcrossScopes,
    MixedRelationships,
    IncomparableEvidence,
    InsufficientMappedScopes,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalRawObservationScopeV0 {
    pub scope_version: String,
    pub provider_id: String,
    pub series_id: String,
    pub source_snapshot_id: String,
    pub source_snapshot_semantic_digest: String,
    pub segmentation_policy_digest: String,
    pub canonical_row_set_digest: String,
    pub canonical_row_order_digest: String,
    pub row_count: usize,
    pub first_timestamp: u64,
    pub last_timestamp: u64,
    pub information_cutoff_timestamp: u64,
    pub scope_kind: CanonicalObservationScopeKindV0,
    pub scope_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalScopeLineageProofV0 {
    pub left_snapshot_id: String,
    pub right_snapshot_id: String,
    pub canonical_row_set_equal: bool,
    pub canonical_row_order_equal: bool,
    pub series_equal: bool,
    pub provider_equal: bool,
    pub information_cutoff_equal: bool,
    pub lineage_status: CanonicalScopeLineageStatusV0,
    pub proof_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentDerivedEvidenceScopeV0 {
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub canonical_raw_scope_digest: String,
    pub feature_history_policy_digest: String,
    pub effective_anchor_set_digest: String,
    pub effective_anchor_count: usize,
    pub sequence_policy_digest: String,
    pub label_policy_digest: String,
    pub source_report_digest: String,
    pub derived_scope_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectiveForecastScopeV0 {
    pub objective: LearnedAgentObjectiveV0,
    pub horizon_policy_digest: String,
    pub horizon_rows: usize,
    pub label_definition_digest: String,
    pub support_policy_digest: String,
    pub objective_scope_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentOpinionScopeAttestationV0 {
    pub attestation_version: String,
    pub opinion_id: String,
    pub opinion_digest: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub canonical_raw_scope_digest: String,
    pub agent_derived_scope_digest: String,
    pub objective_forecast_scope_digest: String,
    pub source_evidence_digest: String,
    pub source_report_digest: String,
    pub attestation_status: OpinionScopeAttestationStatusV0,
    pub reason_codes: Vec<String>,
    pub attestation_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalLearnedAgentScopePairV0 {
    pub pair_id: String,
    pub left_opinion_id: String,
    pub right_opinion_id: String,
    pub left_attestation_digest: String,
    pub right_attestation_digest: String,
    pub raw_scope_alignment: RawObservationScopeAlignmentV0,
    pub effective_sample_alignment: EffectiveSampleAlignmentV0,
    pub horizon_alignment: ObjectiveHorizonAlignmentV0,
    pub comparability: LearnedAgentScopeComparabilityV0,
    pub reason_codes: Vec<String>,
    pub pair_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentScopeMappingRegistryV0 {
    pub registry_version: String,
    pub canonical_scopes: Vec<CanonicalRawObservationScopeV0>,
    pub opinion_attestations: Vec<LearnedAgentOpinionScopeAttestationV0>,
    pub scope_pairs: Vec<CanonicalLearnedAgentScopePairV0>,
    pub unmatched_opinion_ids: Vec<String>,
    pub ambiguous_opinion_ids: Vec<String>,
    pub mapping_status: ScopeMappingRegistryStatusV0,
    pub registry_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrossScopeShadowDeliberationAggregateV0 {
    pub aggregate_id: String,
    pub mapped_scope_count: usize,
    pub unmatched_scope_count: usize,
    pub both_abstained_count: usize,
    pub incomparable_count: usize,
    pub composition_status: AggregateDeliberationCompositionStatusV0,
    pub relationship_summary: AggregateRelationshipSummaryV0,
    pub no_winner_selected: bool,
    pub no_action_selected: bool,
    pub aggregate_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeAlignmentReportV0 {
    pub report_version: String,
    pub offline: bool,
    pub provider_calls: usize,
    pub transport_constructions: usize,
    pub network_consent_reads: usize,
    pub registry: LearnedAgentScopeMappingRegistryV0,
    pub aggregate: CrossScopeShadowDeliberationAggregateV0,
    pub existing_transcript_digests: Vec<String>,
    pub existing_ledger_digest: String,
    pub report_digest: String,
}

fn digest<T: std::fmt::Debug>(value: &T) -> String {
    stable_hash_string(&format!("{value:?}"))
}

fn canonical_scope(
    snapshot: &DataSnapshot,
    start: usize,
    end: usize,
    segmentation_policy_digest: String,
) -> Result<CanonicalRawObservationScopeV0, String> {
    let rows = snapshot
        .normalized_dataset
        .rows
        .get(start..end)
        .ok_or("invalid_scope_range")?;
    if rows.is_empty()
        || rows
            .windows(2)
            .any(|pair| pair[0].timestamp_ms >= pair[1].timestamp_ms)
    {
        return Err("invalid_canonical_rows".into());
    }
    let ordered = rows
        .iter()
        .map(|row| {
            format!(
                "{}:{}:{:x}:{:x}:{:x}:{:x}:{:x}:{:?}",
                row.symbol,
                row.timestamp_ms,
                row.open.to_bits(),
                row.high.to_bits(),
                row.low.to_bits(),
                row.close.to_bits(),
                row.volume.to_bits(),
                row.trade_value.map(f64::to_bits)
            )
        })
        .collect::<Vec<_>>();
    let mut sorted = ordered.clone();
    sorted.sort();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("duplicate_canonical_row".into());
    }
    let canonical_row_set_digest = digest(&sorted);
    let canonical_row_order_digest = digest(&ordered);
    let mut scope = CanonicalRawObservationScopeV0 {
        scope_version: "canonical-raw-observation-scope-v0".into(),
        provider_id: snapshot.normalized_dataset.source.clone(),
        series_id: snapshot.normalized_dataset.symbol.clone(),
        source_snapshot_id: snapshot.snapshot_id.clone(),
        source_snapshot_semantic_digest: snapshot.content_digest.clone(),
        segmentation_policy_digest,
        canonical_row_set_digest,
        canonical_row_order_digest,
        row_count: rows.len(),
        first_timestamp: rows[0].timestamp_ms,
        last_timestamp: rows[rows.len() - 1].timestamp_ms,
        information_cutoff_timestamp: rows[rows.len() - 1].timestamp_ms,
        scope_kind: CanonicalObservationScopeKindV0::HistoricalRegimePack,
        scope_digest: String::new(),
    };
    scope.scope_digest = digest(&(
        &scope.provider_id,
        &scope.series_id,
        &scope.source_snapshot_semantic_digest,
        &scope.segmentation_policy_digest,
        &scope.canonical_row_set_digest,
        &scope.canonical_row_order_digest,
        scope.information_cutoff_timestamp,
    ));
    Ok(scope)
}

pub fn canonical_scope_lineage_proof_v0(
    left: &CanonicalRawObservationScopeV0,
    right: &CanonicalRawObservationScopeV0,
) -> CanonicalScopeLineageProofV0 {
    let canonical_row_set_equal = left.canonical_row_set_digest == right.canonical_row_set_digest;
    let canonical_row_order_equal =
        left.canonical_row_order_digest == right.canonical_row_order_digest;
    let series_equal = left.series_id == right.series_id;
    let provider_equal = left.provider_id == right.provider_id;
    let information_cutoff_equal =
        left.information_cutoff_timestamp == right.information_cutoff_timestamp;
    let lineage_status = if !series_equal {
        CanonicalScopeLineageStatusV0::DifferentSeries
    } else if !provider_equal {
        CanonicalScopeLineageStatusV0::DifferentProvider
    } else if !information_cutoff_equal {
        CanonicalScopeLineageStatusV0::DifferentInformationCutoff
    } else if !canonical_row_set_equal || !canonical_row_order_equal {
        CanonicalScopeLineageStatusV0::DifferentRows
    } else if left.source_snapshot_id == right.source_snapshot_id {
        CanonicalScopeLineageStatusV0::SameSnapshot
    } else {
        CanonicalScopeLineageStatusV0::CertifiedEquivalentRows
    };
    let mut proof = CanonicalScopeLineageProofV0 {
        left_snapshot_id: left.source_snapshot_id.clone(),
        right_snapshot_id: right.source_snapshot_id.clone(),
        canonical_row_set_equal,
        canonical_row_order_equal,
        series_equal,
        provider_equal,
        information_cutoff_equal,
        lineage_status,
        proof_digest: String::new(),
    };
    proof.proof_digest = digest(&(
        &proof.left_snapshot_id,
        &proof.right_snapshot_id,
        proof.canonical_row_set_equal,
        proof.canonical_row_order_equal,
        proof.series_equal,
        proof.provider_equal,
        proof.information_cutoff_equal,
        proof.lineage_status,
    ));
    proof
}

fn derived_scope(
    agent_id: &str,
    objective: LearnedAgentObjectiveV0,
    raw: &CanonicalRawObservationScopeV0,
    history: usize,
    horizon: usize,
    feature_digest: String,
    sequence_digest: String,
    label_digest: String,
) -> AgentDerivedEvidenceScopeV0 {
    let count = raw
        .row_count
        .saturating_sub(history.saturating_add(horizon));
    let anchors = digest(&(&raw.canonical_row_order_digest, history, horizon, count));
    let mut scope = AgentDerivedEvidenceScopeV0 {
        agent_id: agent_id.into(),
        objective,
        canonical_raw_scope_digest: raw.scope_digest.clone(),
        feature_history_policy_digest: feature_digest,
        effective_anchor_set_digest: anchors,
        effective_anchor_count: count,
        sequence_policy_digest: sequence_digest,
        label_policy_digest: label_digest,
        source_report_digest: raw.source_snapshot_semantic_digest.clone(),
        derived_scope_digest: String::new(),
    };
    scope.derived_scope_digest = digest(&(
        &scope.agent_id,
        scope.objective,
        &scope.canonical_raw_scope_digest,
        &scope.feature_history_policy_digest,
        &scope.effective_anchor_set_digest,
        &scope.sequence_policy_digest,
        &scope.label_policy_digest,
    ));
    scope
}

fn forecast_scope(
    objective: LearnedAgentObjectiveV0,
    horizon: usize,
    label_digest: String,
    support_digest: String,
) -> ObjectiveForecastScopeV0 {
    let mut scope = ObjectiveForecastScopeV0 {
        objective,
        horizon_policy_digest: digest(&(horizon, &label_digest)),
        horizon_rows: horizon,
        label_definition_digest: label_digest,
        support_policy_digest: support_digest,
        objective_scope_digest: String::new(),
    };
    scope.objective_scope_digest = digest(&(
        scope.objective,
        &scope.horizon_policy_digest,
        &scope.label_definition_digest,
        &scope.support_policy_digest,
    ));
    scope
}

fn attestation(
    opinion_id: String,
    opinion_digest: String,
    agent_id: String,
    objective: LearnedAgentObjectiveV0,
    source_evidence_digest: String,
    raw: &CanonicalRawObservationScopeV0,
    derived: &AgentDerivedEvidenceScopeV0,
    forecast: &ObjectiveForecastScopeV0,
    status: OpinionScopeAttestationStatusV0,
    reason: &str,
) -> LearnedAgentOpinionScopeAttestationV0 {
    let mut value = LearnedAgentOpinionScopeAttestationV0 {
        attestation_version: "learned-agent-opinion-scope-attestation-v0".into(),
        opinion_id,
        opinion_digest,
        agent_id,
        objective,
        canonical_raw_scope_digest: raw.scope_digest.clone(),
        agent_derived_scope_digest: derived.derived_scope_digest.clone(),
        objective_forecast_scope_digest: forecast.objective_scope_digest.clone(),
        source_evidence_digest: source_evidence_digest.clone(),
        source_report_digest: derived.source_report_digest.clone(),
        attestation_status: status,
        reason_codes: vec![reason.into()],
        attestation_digest: String::new(),
    };
    value.attestation_digest = digest(&(
        &value.opinion_id,
        &value.opinion_digest,
        &value.canonical_raw_scope_digest,
        &value.agent_derived_scope_digest,
        &value.objective_forecast_scope_digest,
        value.attestation_status,
        &value.reason_codes,
    ));
    value
}

fn registry(
    scopes: Vec<CanonicalRawObservationScopeV0>,
    attestations: Vec<LearnedAgentOpinionScopeAttestationV0>,
) -> LearnedAgentScopeMappingRegistryV0 {
    let mut pairs = Vec::new();
    let mut unmatched = Vec::new();
    for momentum in attestations
        .iter()
        .filter(|item| item.objective == LearnedAgentObjectiveV0::DirectionalMomentum)
    {
        let candidates = attestations
            .iter()
            .filter(|item| {
                item.objective == LearnedAgentObjectiveV0::DownsideRisk
                    && item.attestation_status == OpinionScopeAttestationStatusV0::Verified
                    && item.canonical_raw_scope_digest == momentum.canonical_raw_scope_digest
            })
            .collect::<Vec<_>>();
        if momentum.attestation_status != OpinionScopeAttestationStatusV0::Verified
            || candidates.len() != 1
        {
            unmatched.push(momentum.opinion_id.clone());
            continue;
        }
        let risk = candidates[0];
        let effective = if momentum.agent_derived_scope_digest == risk.agent_derived_scope_digest {
            EffectiveSampleAlignmentV0::ExactSameAnchorSets
        } else {
            EffectiveSampleAlignmentV0::SameRawScopeDifferentAnchorSets
        };
        let horizon = ObjectiveHorizonAlignmentV0::DifferentHorizonCompatibleForRegimeSummary;
        let comparability = if effective == EffectiveSampleAlignmentV0::ExactSameAnchorSets {
            LearnedAgentScopeComparabilityV0::ExactDecisionScopeComparable
        } else {
            LearnedAgentScopeComparabilityV0::RegimeSummaryComparableWithCaveats
        };
        let mut pair = CanonicalLearnedAgentScopePairV0 {
            pair_id: format!(
                "scope-pair-{}",
                digest(&(&momentum.attestation_digest, &risk.attestation_digest))
            ),
            left_opinion_id: momentum.opinion_id.clone(),
            right_opinion_id: risk.opinion_id.clone(),
            left_attestation_digest: momentum.attestation_digest.clone(),
            right_attestation_digest: risk.attestation_digest.clone(),
            raw_scope_alignment: RawObservationScopeAlignmentV0::ExactSameScope,
            effective_sample_alignment: effective,
            horizon_alignment: horizon,
            comparability,
            reason_codes: vec!["canonical_rows_verified_without_name_or_index_matching".into()],
            pair_digest: String::new(),
        };
        pair.pair_digest = digest(&(
            &pair.pair_id,
            pair.raw_scope_alignment,
            pair.effective_sample_alignment,
            pair.horizon_alignment,
            pair.comparability,
        ));
        pairs.push(pair);
    }
    for risk in attestations
        .iter()
        .filter(|item| item.objective == LearnedAgentObjectiveV0::DownsideRisk)
    {
        if !pairs
            .iter()
            .any(|pair| pair.right_opinion_id == risk.opinion_id)
        {
            unmatched.push(risk.opinion_id.clone());
        }
    }
    pairs.sort_by(|left, right| left.pair_id.cmp(&right.pair_id));
    unmatched.sort();
    unmatched.dedup();
    let expected = attestations.len();
    let status = if pairs.is_empty() {
        ScopeMappingRegistryStatusV0::Empty
    } else if unmatched.is_empty() && pairs.len() * 2 == expected {
        if pairs.iter().any(|pair| {
            pair.comparability != LearnedAgentScopeComparabilityV0::ExactDecisionScopeComparable
        }) {
            ScopeMappingRegistryStatusV0::FullyMappedWithCaveats
        } else {
            ScopeMappingRegistryStatusV0::FullyMapped
        }
    } else {
        ScopeMappingRegistryStatusV0::PartiallyMapped
    };
    let mut value = LearnedAgentScopeMappingRegistryV0 {
        registry_version: "learned-agent-scope-mapping-registry-v0".into(),
        canonical_scopes: scopes,
        opinion_attestations: attestations,
        scope_pairs: pairs,
        unmatched_opinion_ids: unmatched,
        ambiguous_opinion_ids: vec![],
        mapping_status: status,
        registry_digest: String::new(),
    };
    value.registry_digest = digest(&(
        &value.registry_version,
        &value.canonical_scopes,
        &value.opinion_attestations,
        &value.scope_pairs,
        &value.unmatched_opinion_ids,
        value.mapping_status,
    ));
    value
}

pub fn replay_btc_scope_alignment_v0(
    snapshot: &DataSnapshot,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<ScopeAlignmentReportV0, String> {
    let replays = replay_btc_shadow_deliberations_v0(snapshot, campaign)?;
    let sufficiency = assess_momentum_campaign_sufficiency_v0(snapshot.row_count, campaign)
        .map_err(|_| "momentum_sufficiency_invalid")?;
    let config = BtcHistoricalRegimeConfigV0 {
        minimum_regimes: 2,
        regime_rows: sufficiency.required_minimum_rows,
        inter_regime_gap_rows: campaign.purge_gap_rows,
        minimum_campaign_windows_per_regime: campaign.minimum_evaluated_windows,
        segmentation_policy: TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
    };
    let segmentation = segment_btc_historical_regimes_v0(snapshot, &config)
        .map_err(|_| "momentum_scope_segmentation_failed")?;
    let momentum_raw = segmentation
        .regimes
        .iter()
        .map(|regime| {
            canonical_scope(
                snapshot,
                regime.start_row_index,
                regime.end_row_index_exclusive,
                segmentation.segmentation_config_digest.clone(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let risk_config = CycleRiskShadowConfigV0::default();
    let mid = snapshot.normalized_dataset.rows.len() / 2;
    let risk_raw = vec![
        canonical_scope(snapshot, 0, mid, risk_config.digest())?,
        canonical_scope(
            snapshot,
            mid,
            snapshot.normalized_dataset.rows.len(),
            risk_config.digest(),
        )?,
    ];
    let momentum_history = campaign
        .feature_config
        .momentum_lookback
        .max(campaign.feature_config.trend_lookback)
        .max(campaign.feature_config.volatility_lookback)
        .max(campaign.feature_config.volume_lookback)
        .max(campaign.feature_config.drawdown_lookback)
        + campaign.sequence_config.sequence_length
        - 1;
    let momentum_forecast = forecast_scope(
        LearnedAgentObjectiveV0::DirectionalMomentum,
        campaign.sequence_config.prediction_horizon,
        digest(&campaign.sequence_config),
        digest(&campaign.support_gate),
    );
    let risk_forecast = forecast_scope(
        LearnedAgentObjectiveV0::DownsideRisk,
        risk_config.label.horizon_rows,
        risk_config.label.digest(),
        risk_config.feature.digest(),
    );
    let mut attestations = Vec::new();
    for replay in &replays {
        let Some(regime_id) = replay.momentum.temporal_scope.regime_id.as_ref() else {
            continue;
        };
        let Some(regime) = segmentation
            .regimes
            .iter()
            .find(|regime| regime.regime_id == *regime_id)
        else {
            continue;
        };
        let raw = canonical_scope(
            snapshot,
            regime.start_row_index,
            regime.end_row_index_exclusive,
            segmentation.segmentation_config_digest.clone(),
        )?;
        let derived = derived_scope(
            &replay.momentum.agent_id,
            LearnedAgentObjectiveV0::DirectionalMomentum,
            &raw,
            momentum_history,
            campaign.sequence_config.prediction_horizon,
            digest(&campaign.feature_config),
            digest(&campaign.sequence_config),
            digest(&campaign.training_config),
        );
        attestations.push(attestation(
            replay.momentum.opinion_id.clone(),
            replay.momentum.opinion_digest.clone(),
            replay.momentum.agent_id.clone(),
            LearnedAgentObjectiveV0::DirectionalMomentum,
            replay.momentum.source_evidence_digest.clone(),
            &raw,
            &derived,
            &momentum_forecast,
            OpinionScopeAttestationStatusV0::Verified,
            "momentum_scope_reconstructed_from_immutable_regime_rows",
        ));
    }
    for replay in &replays {
        if replay.risk.temporal_scope.regime_id.is_none() {
            continue;
        }
        let mut unresolved = LearnedAgentOpinionScopeAttestationV0 {
            attestation_version: "learned-agent-opinion-scope-attestation-v0".into(),
            opinion_id: replay.risk.opinion_id.clone(),
            opinion_digest: replay.risk.opinion_digest.clone(),
            agent_id: replay.risk.agent_id.clone(),
            objective: LearnedAgentObjectiveV0::DownsideRisk,
            canonical_raw_scope_digest: String::new(),
            agent_derived_scope_digest: String::new(),
            objective_forecast_scope_digest: risk_forecast.objective_scope_digest.clone(),
            source_evidence_digest: replay.risk.source_evidence_digest.clone(),
            source_report_digest: String::new(),
            attestation_status: OpinionScopeAttestationStatusV0::AmbiguousSource,
            reason_codes: vec![
                "existing_risk_opinion_does_not_identify_one_reconstructed_risk_scope".into(),
            ],
            attestation_digest: String::new(),
        };
        unresolved.attestation_digest = digest(&(
            &unresolved.opinion_id,
            &unresolved.opinion_digest,
            unresolved.attestation_status,
            &unresolved.reason_codes,
        ));
        attestations.push(unresolved);
    }
    let momentum_scope_count = momentum_raw.len();
    let mut scopes = momentum_raw;
    scopes.extend(risk_raw);
    scopes.sort_by(|left, right| left.scope_digest.cmp(&right.scope_digest));
    scopes.dedup_by(|left, right| left.scope_digest == right.scope_digest);
    let registry = registry(scopes, attestations);
    let composition_status = match registry.mapping_status {
        ScopeMappingRegistryStatusV0::FullyMapped => {
            AggregateDeliberationCompositionStatusV0::FullyComposed
        }
        ScopeMappingRegistryStatusV0::FullyMappedWithCaveats => {
            AggregateDeliberationCompositionStatusV0::FullyComposedWithScopeCaveats
        }
        ScopeMappingRegistryStatusV0::PartiallyMapped => {
            AggregateDeliberationCompositionStatusV0::PartiallyMappedNotComposable
        }
        ScopeMappingRegistryStatusV0::Ambiguous => {
            AggregateDeliberationCompositionStatusV0::AmbiguousNotComposable
        }
        ScopeMappingRegistryStatusV0::Empty => {
            AggregateDeliberationCompositionStatusV0::EmptyNotComposable
        }
        ScopeMappingRegistryStatusV0::Invalid => AggregateDeliberationCompositionStatusV0::Invalid,
    };
    let relationships = replays
        .iter()
        .take(momentum_scope_count)
        .map(|replay| replay.relationship.relationship)
        .collect::<Vec<_>>();
    let both_abstained_count = relationships
        .iter()
        .filter(|value| **value == AgentOpinionRelationshipV0::BothAbstained)
        .count();
    let aggregate = CrossScopeShadowDeliberationAggregateV0 {
        aggregate_id: format!("scope-aggregate-{}", registry.registry_digest),
        mapped_scope_count: registry.scope_pairs.len(),
        unmatched_scope_count: registry.unmatched_opinion_ids.len(),
        both_abstained_count,
        incomparable_count: 0,
        composition_status,
        relationship_summary: if registry.scope_pairs.is_empty() {
            AggregateRelationshipSummaryV0::InsufficientMappedScopes
        } else if both_abstained_count == registry.scope_pairs.len() {
            AggregateRelationshipSummaryV0::AllScopesBothAbstained
        } else {
            AggregateRelationshipSummaryV0::MixedRelationships
        },
        no_winner_selected: true,
        no_action_selected: true,
        aggregate_digest: String::new(),
    };
    let mut aggregate = aggregate;
    aggregate.aggregate_digest = digest(&(
        &aggregate.aggregate_id,
        &registry.registry_digest,
        aggregate.mapped_scope_count,
        aggregate.unmatched_scope_count,
        aggregate.both_abstained_count,
        aggregate.composition_status,
    ));
    let mut ledger = new_shadow_deliberation_ledger_v0();
    for replay in replays.iter().take(momentum_scope_count) {
        append_shadow_deliberation_v0(&mut ledger, replay)
            .map_err(|_| "existing_shadow_ledger_integrity_failed")?;
    }
    let mut value = ScopeAlignmentReportV0 {
        report_version: "learned-agent-scope-alignment-v0".into(),
        offline: true,
        provider_calls: 0,
        transport_constructions: 0,
        network_consent_reads: 0,
        registry,
        aggregate,
        existing_transcript_digests: replays
            .iter()
            .take(momentum_scope_count)
            .map(|replay| replay.transcript.transcript_digest.clone())
            .collect(),
        existing_ledger_digest: ledger.ledger_digest,
        report_digest: String::new(),
    };
    value.report_digest = digest(&(
        &value.registry.registry_digest,
        &value.aggregate.aggregate_digest,
        &value.existing_transcript_digests,
    ));
    Ok(value)
}

// V1 deliberately lives beside the V0 report.  It is an external audit layer:
// it never feeds a model, rewrites a legacy digest, or changes a partition.
pub trait CanonicalSemanticEncodeV1 {
    fn encode_canonical_v1(&self, out: &mut Vec<u8>);
}

pub fn canonical_semantic_digest_v1<T: CanonicalSemanticEncodeV1>(value: &T) -> String {
    let mut bytes = Vec::new();
    value.encode_canonical_v1(&mut bytes);
    stable_hash_string(&hex(&bytes))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        value.push(DIGITS[(byte >> 4) as usize] as char);
        value.push(DIGITS[(byte & 15) as usize] as char);
    }
    value
}
fn tag(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}
fn u64v(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}
fn usizev(out: &mut Vec<u8>, value: usize) {
    u64v(out, value as u64);
}
fn boolv(out: &mut Vec<u8>, value: bool) {
    tag(out, u8::from(value));
}
fn strv(out: &mut Vec<u8>, value: &str) {
    usizev(out, value.len());
    out.extend_from_slice(value.as_bytes());
}
fn f32v(out: &mut Vec<u8>, value: f32) {
    u64v(out, value.to_bits() as u64);
}
fn opt_strv(out: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(v) => {
            tag(out, 1);
            strv(out, v);
        }
        None => tag(out, 0),
    }
}
fn opt_f32v(out: &mut Vec<u8>, value: Option<f32>) {
    match value {
        Some(v) => {
            tag(out, 1);
            f32v(out, v);
        }
        None => tag(out, 0),
    }
}
fn strings(out: &mut Vec<u8>, values: &[String]) {
    usizev(out, values.len());
    for value in values {
        strv(out, value);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyScopeReferenceV1 {
    pub legacy_protocol_version: String,
    pub legacy_scope_digest: Option<String>,
    pub legacy_attestation_digest: Option<String>,
    pub legacy_registry_digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalHistoricalRowIdentityV1 {
    pub provider_id: String,
    pub series_id: String,
    pub timestamp_ms: u64,
    pub open_bits: u64,
    pub high_bits: u64,
    pub low_bits: u64,
    pub close_bits: u64,
    pub volume_bits: u64,
    pub trade_value_bits: Option<u64>,
    pub row_digest_v1: String,
}
impl CanonicalSemanticEncodeV1 for CanonicalHistoricalRowIdentityV1 {
    fn encode_canonical_v1(&self, out: &mut Vec<u8>) {
        strv(out, "canonical-historical-row-v1");
        strv(out, &self.provider_id);
        strv(out, &self.series_id);
        u64v(out, self.timestamp_ms);
        u64v(out, self.open_bits);
        u64v(out, self.high_bits);
        u64v(out, self.low_bits);
        u64v(out, self.close_bits);
        u64v(out, self.volume_bits);
        match self.trade_value_bits {
            Some(value) => {
                tag(out, 1);
                u64v(out, value);
            }
            None => tag(out, 0),
        }
    }
}
fn row_identity_v1(
    snapshot: &DataSnapshot,
    row: &crate::league::HistoricalOhlcvRow,
) -> Result<CanonicalHistoricalRowIdentityV1, String> {
    if ![row.open, row.high, row.low, row.close, row.volume]
        .iter()
        .all(|value| value.is_finite())
        || row.trade_value.is_some_and(|value| !value.is_finite())
    {
        return Err("non_finite_canonical_row".into());
    }
    let mut value = CanonicalHistoricalRowIdentityV1 {
        provider_id: snapshot.normalized_dataset.source.clone(),
        series_id: row.symbol.clone(),
        timestamp_ms: row.timestamp_ms,
        open_bits: row.open.to_bits(),
        high_bits: row.high.to_bits(),
        low_bits: row.low.to_bits(),
        close_bits: row.close.to_bits(),
        volume_bits: row.volume.to_bits(),
        trade_value_bits: row.trade_value.map(f64::to_bits),
        row_digest_v1: String::new(),
    };
    value.row_digest_v1 = canonical_semantic_digest_v1(&value);
    Ok(value)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalRawObservationScopeV1 {
    pub scope_version: String,
    pub provider_id: String,
    pub series_id: String,
    pub source_snapshot_id: String,
    pub source_snapshot_semantic_digest: String,
    pub range_start_index: usize,
    pub range_end_index_exclusive: usize,
    pub row_identity_digests: Vec<String>,
    pub canonical_row_set_digest_v1: String,
    pub canonical_row_order_digest_v1: String,
    pub row_count: usize,
    pub first_timestamp: u64,
    pub last_timestamp: u64,
    pub information_cutoff_timestamp: u64,
    pub segmentation_policy_digest: String,
    pub scope_digest_v1: String,
}
impl CanonicalSemanticEncodeV1 for CanonicalRawObservationScopeV1 {
    fn encode_canonical_v1(&self, out: &mut Vec<u8>) {
        strv(out, &self.scope_version);
        strv(out, &self.provider_id);
        strv(out, &self.series_id);
        strv(out, &self.source_snapshot_id);
        strv(out, &self.source_snapshot_semantic_digest);
        usizev(out, self.range_start_index);
        usizev(out, self.range_end_index_exclusive);
        strings(out, &self.row_identity_digests);
        strv(out, &self.canonical_row_set_digest_v1);
        strv(out, &self.canonical_row_order_digest_v1);
        usizev(out, self.row_count);
        u64v(out, self.first_timestamp);
        u64v(out, self.last_timestamp);
        u64v(out, self.information_cutoff_timestamp);
        strv(out, &self.segmentation_policy_digest);
    }
}
pub fn canonical_raw_scope_v1(
    snapshot: &DataSnapshot,
    start: usize,
    end: usize,
    policy: &str,
) -> Result<CanonicalRawObservationScopeV1, String> {
    let rows = snapshot
        .normalized_dataset
        .rows
        .get(start..end)
        .ok_or("invalid_v1_scope_range")?;
    if rows.is_empty()
        || rows
            .windows(2)
            .any(|pair| pair[0].timestamp_ms >= pair[1].timestamp_ms)
    {
        return Err("invalid_v1_scope_rows".into());
    }
    let identities = rows
        .iter()
        .map(|row| row_identity_v1(snapshot, row))
        .collect::<Result<Vec<_>, _>>()?;
    let order = identities
        .iter()
        .map(|value| value.row_digest_v1.clone())
        .collect::<Vec<_>>();
    let mut set = order.clone();
    set.sort();
    if set.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("duplicate_v1_canonical_row".into());
    }
    let mut value = CanonicalRawObservationScopeV1 {
        scope_version: "canonical-raw-observation-scope-v1".into(),
        provider_id: snapshot.normalized_dataset.source.clone(),
        series_id: snapshot.normalized_dataset.symbol.clone(),
        source_snapshot_id: snapshot.snapshot_id.clone(),
        source_snapshot_semantic_digest: snapshot.content_digest.clone(),
        range_start_index: start,
        range_end_index_exclusive: end,
        row_identity_digests: order.clone(),
        canonical_row_set_digest_v1: strings_digest_v1("row-set-v1", &set),
        canonical_row_order_digest_v1: strings_digest_v1("row-order-v1", &order),
        row_count: rows.len(),
        first_timestamp: rows[0].timestamp_ms,
        last_timestamp: rows[rows.len() - 1].timestamp_ms,
        information_cutoff_timestamp: rows[rows.len() - 1].timestamp_ms,
        segmentation_policy_digest: policy.into(),
        scope_digest_v1: String::new(),
    };
    value.scope_digest_v1 = canonical_semantic_digest_v1(&value);
    Ok(value)
}
fn strings_digest_v1(domain: &str, values: &[String]) -> String {
    let mut bytes = Vec::new();
    strv(&mut bytes, domain);
    strings(&mut bytes, values);
    stable_hash_string(&hex(&bytes))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CycleRiskHistoricalRangeCandidateV0 {
    pub candidate_range_id: String,
    pub start_row_index: usize,
    pub end_row_index_exclusive: usize,
    pub row_count: usize,
    pub canonical_scope_digest_v1: String,
    pub expected_frozen_pack_digest: String,
    pub range_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CycleRiskHistoricalRangePlanV0 {
    pub source_snapshot_id: String,
    pub source_snapshot_digest: String,
    pub config_digest: String,
    pub ranges: Vec<CycleRiskHistoricalRangeCandidateV0>,
    pub plan_digest: String,
}
fn risk_config_digest_v1(config: &CycleRiskShadowConfigV0) -> String {
    let mut bytes = Vec::new();
    strv(&mut bytes, "cycle-risk-config-v1");
    usizev(&mut bytes, config.feature.short_lookback);
    usizev(&mut bytes, config.feature.long_lookback);
    usizev(&mut bytes, config.feature.drawdown_lookback);
    f32v(&mut bytes, config.feature.epsilon);
    usizev(&mut bytes, config.label.horizon_rows);
    strv(&mut bytes, &config.label.threshold_policy);
    f32v(&mut bytes, config.label.training_quantile);
    usizev(&mut bytes, config.label.minimum_training_anchors);
    usizev(&mut bytes, config.label.minimum_positive_labels);
    usizev(&mut bytes, config.label.minimum_negative_labels);
    usizev(&mut bytes, config.label.purge_gap_rows);
    f32v(&mut bytes, config.label.epsilon);
    usizev(&mut bytes, config.sequence_length);
    f32v(&mut bytes, config.train_fraction);
    f32v(&mut bytes, config.validation_fraction);
    u64v(&mut bytes, config.seed);
    f32v(&mut bytes, config.false_negative_safe_probability);
    usizev(&mut bytes, config.maximum_high_confidence_false_negatives);
    stable_hash_string(&hex(&bytes))
}
pub fn cycle_risk_historical_range_plan_v0(
    snapshot: &DataSnapshot,
    config: &CycleRiskShadowConfigV0,
) -> Result<CycleRiskHistoricalRangePlanV0, String> {
    config
        .validate()
        .map_err(|_| "risk_range_plan_invalid_config")?;
    let snapshot_digest =
        crate::data::historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
    if snapshot_digest != snapshot.content_digest {
        return Err("risk_range_plan_snapshot_digest_mismatch".into());
    }
    let mid = snapshot.normalized_dataset.rows.len() / 2;
    let mut ranges = Vec::new();
    for (start, end) in [(0usize, mid), (mid, snapshot.normalized_dataset.rows.len())] {
        let scope = canonical_raw_scope_v1(snapshot, start, end, &risk_config_digest_v1(config))?;
        let pack = stable_hash_string(&format!(
            "cycle-risk-pack-v0:{}:{}:{}:{}",
            snapshot.snapshot_id, start, end, snapshot_digest
        ));
        let mut bytes = Vec::new();
        strv(&mut bytes, "cycle-risk-range-candidate-v1");
        usizev(&mut bytes, start);
        usizev(&mut bytes, end);
        strv(&mut bytes, &scope.scope_digest_v1);
        strv(&mut bytes, &pack);
        let range_digest = stable_hash_string(&hex(&bytes));
        ranges.push(CycleRiskHistoricalRangeCandidateV0 {
            candidate_range_id: format!("risk-range-{}", range_digest),
            start_row_index: start,
            end_row_index_exclusive: end,
            row_count: end - start,
            canonical_scope_digest_v1: scope.scope_digest_v1,
            expected_frozen_pack_digest: pack,
            range_digest,
        });
    }
    let mut bytes = Vec::new();
    strv(&mut bytes, "cycle-risk-range-plan-v1");
    strv(&mut bytes, &snapshot.snapshot_id);
    strv(&mut bytes, &snapshot_digest);
    strv(&mut bytes, &risk_config_digest_v1(config));
    strings(
        &mut bytes,
        &ranges
            .iter()
            .map(|value| value.range_digest.clone())
            .collect::<Vec<_>>(),
    );
    Ok(CycleRiskHistoricalRangePlanV0 {
        source_snapshot_id: snapshot.snapshot_id.clone(),
        source_snapshot_digest: snapshot_digest,
        config_digest: risk_config_digest_v1(config),
        ranges,
        plan_digest: stable_hash_string(&hex(&bytes)),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleRiskRangeResolutionStatusV0 {
    VerifiedUniqueMatch,
    NoMatchingRange,
    MultipleMatchingRanges,
    SnapshotMismatch,
    PackDigestMismatch,
    InvalidRangePlan,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CycleRiskRangeResolutionV1 {
    pub status: CycleRiskRangeResolutionStatusV0,
    pub candidate_scope_digest_v1: Option<String>,
    pub candidate_range_id: Option<String>,
}
fn resolve_risk_range(
    plan: &CycleRiskHistoricalRangePlanV0,
    result: &super::CycleRiskRegimeResultV0,
) -> CycleRiskRangeResolutionV1 {
    if result.source_snapshot_id != plan.source_snapshot_id {
        return CycleRiskRangeResolutionV1 {
            status: CycleRiskRangeResolutionStatusV0::SnapshotMismatch,
            candidate_scope_digest_v1: None,
            candidate_range_id: None,
        };
    }
    let matches = plan
        .ranges
        .iter()
        .filter(|value| value.expected_frozen_pack_digest == result.frozen_pack_digest)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [value] => CycleRiskRangeResolutionV1 {
            status: CycleRiskRangeResolutionStatusV0::VerifiedUniqueMatch,
            candidate_scope_digest_v1: Some(value.canonical_scope_digest_v1.clone()),
            candidate_range_id: Some(value.candidate_range_id.clone()),
        },
        [] => CycleRiskRangeResolutionV1 {
            status: CycleRiskRangeResolutionStatusV0::NoMatchingRange,
            candidate_scope_digest_v1: None,
            candidate_range_id: None,
        },
        _ => CycleRiskRangeResolutionV1 {
            status: CycleRiskRangeResolutionStatusV0::MultipleMatchingRanges,
            candidate_scope_digest_v1: None,
            candidate_range_id: None,
        },
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CycleRiskRegimeResultIdentityV1 {
    pub agent_id: String,
    pub source_snapshot_id: String,
    pub source_snapshot_digest: String,
    pub frozen_pack_digest: String,
    pub canonical_scope_digest_v1: String,
    pub config_digest: String,
    pub feature_config_digest: String,
    pub label_config_digest: String,
    pub sequence_policy_digest: String,
    pub checkpoint_identity_digest: String,
    pub verdict_tag: String,
    pub accepted_model_version_id: Option<String>,
    pub result_digest_v1: String,
}
fn metric_identity_v1(metric: &super::CycleRiskMetricSetV0, out: &mut Vec<u8>) {
    f32v(out, metric.brier);
    f32v(out, metric.calibration_reliability);
    f32v(out, metric.resolution);
    f32v(out, metric.uncertainty);
    opt_f32v(out, metric.rank_auc);
    f32v(out, metric.prevalence);
    f32v(out, metric.mean_probability);
    f32v(out, metric.probability_stddev);
    f32v(out, metric.coverage);
    usizev(out, metric.abstain_count);
    usizev(out, metric.high_confidence_false_negatives);
    usizev(out, metric.high_confidence_false_positives);
    boolv(out, metric.probability_collapse);
}
fn verdict_tag_v1(value: super::CycleRiskShadowVerdictV0) -> &'static str {
    match value {
        super::CycleRiskShadowVerdictV0::PositiveEvidence => "positive_evidence",
        super::CycleRiskShadowVerdictV0::LinearBaselineStronger => "linear_baseline_stronger",
        super::CycleRiskShadowVerdictV0::ConstantBaselineStronger => "constant_baseline_stronger",
        super::CycleRiskShadowVerdictV0::ProbabilityCollapse => "probability_collapse",
        super::CycleRiskShadowVerdictV0::HighConfidenceFalseNegative => {
            "high_confidence_false_negative"
        }
        super::CycleRiskShadowVerdictV0::InsufficientEvents => "insufficient_events",
        super::CycleRiskShadowVerdictV0::ShadowOnly => "shadow_only",
    }
}
fn risk_result_identity_v1(
    report: &super::CycleRiskShadowReportV0,
    result: &super::CycleRiskRegimeResultV0,
    config: &CycleRiskShadowConfigV0,
    resolution: &CycleRiskRangeResolutionV1,
) -> Option<CycleRiskRegimeResultIdentityV1> {
    let scope = resolution.candidate_scope_digest_v1.clone()?;
    let mut checkpoint = Vec::new();
    strv(&mut checkpoint, "cycle-risk-checkpoint-v1");
    f32v(&mut checkpoint, result.checkpoint.threshold);
    metric_identity_v1(&result.checkpoint.r0, &mut checkpoint);
    metric_identity_v1(&result.checkpoint.r1, &mut checkpoint);
    metric_identity_v1(&result.checkpoint.r2, &mut checkpoint);
    metric_identity_v1(&result.checkpoint.train, &mut checkpoint);
    metric_identity_v1(&result.checkpoint.validation, &mut checkpoint);
    metric_identity_v1(&result.checkpoint.test, &mut checkpoint);
    boolv(&mut checkpoint, result.checkpoint.test_sealed_once);
    opt_strv(
        &mut checkpoint,
        result.checkpoint.accepted_model_version.as_deref(),
    );
    let checkpoint_identity_digest = stable_hash_string(&hex(&checkpoint));
    let mut bytes = Vec::new();
    strv(&mut bytes, "cycle-risk-regime-result-v1");
    strv(&mut bytes, &report.agent_id);
    strv(&mut bytes, &report.snapshot_id);
    strv(&mut bytes, &report.snapshot_digest);
    strv(&mut bytes, &result.frozen_pack_digest);
    strv(&mut bytes, &scope);
    strv(&mut bytes, &risk_config_digest_v1(config));
    strv(&mut bytes, &config.feature.digest());
    strv(&mut bytes, &config.label.digest());
    usizev(&mut bytes, config.sequence_length);
    strv(&mut bytes, &checkpoint_identity_digest);
    strv(&mut bytes, verdict_tag_v1(result.verdict));
    opt_strv(
        &mut bytes,
        result.checkpoint.accepted_model_version.as_deref(),
    );
    Some(CycleRiskRegimeResultIdentityV1 {
        agent_id: report.agent_id.clone(),
        source_snapshot_id: report.snapshot_id.clone(),
        source_snapshot_digest: report.snapshot_digest.clone(),
        frozen_pack_digest: result.frozen_pack_digest.clone(),
        canonical_scope_digest_v1: scope,
        config_digest: risk_config_digest_v1(config),
        feature_config_digest: config.feature.digest(),
        label_config_digest: config.label.digest(),
        sequence_policy_digest: strings_digest_v1(
            "risk-sequence-policy-v1",
            &[config.sequence_length.to_string()],
        ),
        checkpoint_identity_digest,
        verdict_tag: verdict_tag_v1(result.verdict).into(),
        accepted_model_version_id: result.checkpoint.accepted_model_version.clone(),
        result_digest_v1: stable_hash_string(&hex(&bytes)),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HistoricalPartitionV1 {
    Train,
    Validation,
    Test,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveAnchorIdentityV1 {
    pub anchor_timestamp: u64,
    pub anchor_row_identity_digest: String,
    pub partition: HistoricalPartitionV1,
    pub input_start_timestamp: u64,
    pub input_end_timestamp: u64,
    pub required_label_end_timestamp: u64,
    pub anchor_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentEffectiveAnchorScopeV1 {
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub canonical_raw_scope_digest_v1: String,
    pub train_anchor_digests: Vec<String>,
    pub validation_anchor_digests: Vec<String>,
    pub test_anchor_digests: Vec<String>,
    pub all_anchor_digests: Vec<String>,
    pub train_anchor_set_digest: String,
    pub validation_anchor_set_digest: String,
    pub test_anchor_set_digest: String,
    pub all_anchor_set_digest: String,
    pub all_anchor_order_digest: String,
    pub effective_anchor_count: usize,
    pub scope_digest_v1: String,
}
fn anchor(
    scope: &CanonicalRawObservationScopeV1,
    rows: &[crate::league::HistoricalOhlcvRow],
    anchor_index: usize,
    input_start: usize,
    label_end: usize,
    partition: HistoricalPartitionV1,
) -> EffectiveAnchorIdentityV1 {
    let tag_value = match partition {
        HistoricalPartitionV1::Train => 1,
        HistoricalPartitionV1::Validation => 2,
        HistoricalPartitionV1::Test => 3,
    };
    let mut bytes = Vec::new();
    strv(&mut bytes, "effective-anchor-v1");
    strv(&mut bytes, &scope.row_identity_digests[anchor_index]);
    tag(&mut bytes, tag_value);
    u64v(&mut bytes, rows[input_start].timestamp_ms);
    u64v(&mut bytes, rows[anchor_index].timestamp_ms);
    u64v(&mut bytes, rows[label_end].timestamp_ms);
    let digest = stable_hash_string(&hex(&bytes));
    EffectiveAnchorIdentityV1 {
        anchor_timestamp: rows[anchor_index].timestamp_ms,
        anchor_row_identity_digest: scope.row_identity_digests[anchor_index].clone(),
        partition,
        input_start_timestamp: rows[input_start].timestamp_ms,
        input_end_timestamp: rows[anchor_index].timestamp_ms,
        required_label_end_timestamp: rows[label_end].timestamp_ms,
        anchor_digest: digest,
    }
}
fn anchor_scope(
    agent: &str,
    objective: LearnedAgentObjectiveV0,
    scope: &CanonicalRawObservationScopeV1,
    anchors: Vec<EffectiveAnchorIdentityV1>,
) -> AgentEffectiveAnchorScopeV1 {
    let mut train = Vec::new();
    let mut validation = Vec::new();
    let mut test = Vec::new();
    for anchor in anchors {
        match anchor.partition {
            HistoricalPartitionV1::Train => train.push(anchor.anchor_digest),
            HistoricalPartitionV1::Validation => validation.push(anchor.anchor_digest),
            HistoricalPartitionV1::Test => test.push(anchor.anchor_digest),
        }
    }
    let all = train
        .iter()
        .chain(&validation)
        .chain(&test)
        .cloned()
        .collect::<Vec<_>>();
    let set = |values: &[String]| {
        let mut value = values.to_vec();
        value.sort();
        value.dedup();
        strings_digest_v1("anchor-set-v1", &value)
    };
    let mut bytes = Vec::new();
    strv(&mut bytes, "effective-anchor-scope-v1");
    strv(&mut bytes, agent);
    tag(
        &mut bytes,
        match objective {
            LearnedAgentObjectiveV0::DirectionalMomentum => 1,
            LearnedAgentObjectiveV0::DownsideRisk => 2,
        },
    );
    strv(&mut bytes, &scope.scope_digest_v1);
    strings(&mut bytes, &all);
    AgentEffectiveAnchorScopeV1 {
        agent_id: agent.into(),
        objective,
        canonical_raw_scope_digest_v1: scope.scope_digest_v1.clone(),
        train_anchor_set_digest: set(&train),
        validation_anchor_set_digest: set(&validation),
        test_anchor_set_digest: set(&test),
        all_anchor_set_digest: set(&all),
        all_anchor_order_digest: strings_digest_v1("anchor-order-v1", &all),
        effective_anchor_count: all.len(),
        scope_digest_v1: stable_hash_string(&hex(&bytes)),
        train_anchor_digests: train,
        validation_anchor_digests: validation,
        test_anchor_digests: test,
        all_anchor_digests: all,
    }
}
fn risk_anchor_scope_v1(
    snapshot: &DataSnapshot,
    candidate: &CycleRiskHistoricalRangeCandidateV0,
    config: &CycleRiskShadowConfigV0,
) -> Result<AgentEffectiveAnchorScopeV1, String> {
    let scope = canonical_raw_scope_v1(
        snapshot,
        candidate.start_row_index,
        candidate.end_row_index_exclusive,
        &risk_config_digest_v1(config),
    )?;
    let rows = &snapshot.normalized_dataset.rows
        [candidate.start_row_index..candidate.end_row_index_exclusive];
    let history = config.feature.drawdown_lookback + 1;
    if rows.len() <= history {
        return Err("risk_anchor_history_invalid".into());
    }
    let features = rows.len() - history;
    let train_end = (features as f32 * config.train_fraction).floor() as usize;
    let validation_end =
        train_end + (features as f32 * config.validation_fraction).floor() as usize;
    let gap = config.label.purge_gap_rows + config.sequence_length;
    let partitions = [
        (0, train_end, HistoricalPartitionV1::Train),
        (
            train_end.saturating_add(gap),
            validation_end,
            HistoricalPartitionV1::Validation,
        ),
        (
            validation_end.saturating_add(gap),
            features,
            HistoricalPartitionV1::Test,
        ),
    ];
    let mut anchors = Vec::new();
    for (start, end, partition) in partitions {
        if start >= end {
            continue;
        }
        for feature_end in start.saturating_add(config.sequence_length.saturating_sub(1))..end {
            let raw_anchor = history + feature_end;
            let raw_start = history + feature_end + 1 - config.sequence_length;
            let label = raw_anchor + config.label.horizon_rows;
            if label < rows.len() {
                anchors.push(anchor(
                    &scope, rows, raw_anchor, raw_start, label, partition,
                ));
            }
        }
    }
    Ok(anchor_scope(
        CYCLE_RISK_SHADOW_AGENT_ID_V0,
        LearnedAgentObjectiveV0::DownsideRisk,
        &scope,
        anchors,
    ))
}

pub fn momentum_anchor_scope_v1(
    snapshot: &DataSnapshot,
    start: usize,
    end: usize,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<AgentEffectiveAnchorScopeV1, String> {
    let scope = canonical_raw_scope_v1(snapshot, start, end, &campaign.digest())?;
    let rows = &snapshot.normalized_dataset.rows[start..end];
    let candles = rows
        .iter()
        .map(|row| MomentumCandleV0 {
            timestamp: row.timestamp_ms as i64,
            open: row.open as f32,
            high: row.high as f32,
            low: row.low as f32,
            close: row.close as f32,
            volume: row.volume as f32,
        })
        .collect::<Vec<_>>();
    let features = build_momentum_features_v0(&candles, &campaign.feature_config)
        .map_err(|_| "momentum_anchor_features_invalid")?;
    let snapshot_ids = vec![snapshot.snapshot_id.clone()];
    let examples = build_momentum_sequence_examples_v0(
        &candles,
        &features,
        &campaign.sequence_config,
        &snapshot_ids,
    )
    .map_err(|_| "momentum_anchor_examples_invalid")?;
    let windows = build_momentum_learning_windows_v0(campaign, rows.len(), &snapshot_ids)
        .map_err(|_| "momentum_anchor_windows_invalid")?;
    let mut anchors = Vec::new();
    for window in windows {
        for (range, partition) in [
            (&window.train_range, HistoricalPartitionV1::Train),
            (&window.validation_range, HistoricalPartitionV1::Validation),
            (&window.test_range, HistoricalPartitionV1::Test),
        ] {
            for example in examples.iter().filter(|value| {
                value.sequence_start >= range.start && value.label_index < range.end
            }) {
                anchors.push(anchor(
                    &scope,
                    rows,
                    example.sequence_end,
                    example.sequence_start,
                    example.label_index,
                    partition,
                ));
            }
        }
    }
    Ok(anchor_scope(
        &campaign.agent_id,
        LearnedAgentObjectiveV0::DirectionalMomentum,
        &scope,
        anchors,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleRiskOpinionWitnessStatusV0 {
    VerifiedUniqueSource,
    NoMatchingSource,
    MultipleMatchingSources,
    OpinionSealInvalid,
    ResultIdentityInvalid,
    SnapshotMismatch,
    PackMismatch,
    AdapterReconstructionMismatch,
    TechnicalFailure,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CycleRiskOpinionProvenanceWitnessV0 {
    pub witness_version: String,
    pub opinion_id: String,
    pub opinion_digest: String,
    pub seal_digest: String,
    pub candidate_result_digest_v1: String,
    pub candidate_scope_digest_v1: String,
    pub reconstructed_opinion_digest: String,
    pub reconstructed_matches_existing: bool,
    pub source_snapshot_matches: bool,
    pub pack_identity_matches: bool,
    pub objective_matches: bool,
    pub agent_matches: bool,
    pub witness_status: CycleRiskOpinionWitnessStatusV0,
    pub reason_codes: Vec<String>,
    pub witness_digest_v1: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleRiskProvenanceRegistryStatusV0 {
    FullyVerified,
    PartiallyVerified,
    Ambiguous,
    Empty,
    Invalid,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CycleRiskOpinionProvenanceRegistryV0 {
    pub registry_version: String,
    pub range_plan_digest: String,
    pub result_identities: Vec<CycleRiskRegimeResultIdentityV1>,
    pub witnesses: Vec<CycleRiskOpinionProvenanceWitnessV0>,
    pub unmatched_opinion_ids: Vec<String>,
    pub multiply_matched_opinion_ids: Vec<String>,
    pub status: CycleRiskProvenanceRegistryStatusV0,
    pub registry_digest_v1: String,
}
fn witness_registry_v1(
    report: &super::CycleRiskShadowReportV0,
    plan: &CycleRiskHistoricalRangePlanV0,
    identities: &[CycleRiskRegimeResultIdentityV1],
    existing: &[super::ShadowDeliberationReplayV0],
) -> CycleRiskOpinionProvenanceRegistryV0 {
    let mut witnesses = Vec::new();
    let mut unmatched = Vec::new();
    let mut multiple = Vec::new();
    for replay in existing {
        if replay.risk.opinion_id == "risk-opinion-historical-aggregate" {
            continue;
        }
        let matches = report
            .regimes
            .iter()
            .filter_map(|result| {
                let identity = identities
                    .iter()
                    .find(|value| value.frozen_pack_digest == result.frozen_pack_digest)?;
                let context = CycleRiskOpinionAdapterContextV0 {
                    legacy_regime_id: replay
                        .risk
                        .temporal_scope
                        .regime_id
                        .clone()
                        .unwrap_or_default(),
                };
                let reconstructed =
                    reconstruct_cycle_risk_opinion_from_regime_v0(report, result, &context).ok()?;
                let mut reconstructed = reconstructed;
                let seal = super::learned_agent_opinion::seal_for_provenance_v0(&mut reconstructed);
                let matches = reconstructed.opinion_digest == replay.risk.opinion_digest
                    && seal.seal_digest == replay.risk_seal.seal_digest;
                Some((identity, reconstructed, matches))
            })
            .collect::<Vec<_>>();
        let matching = matches.iter().filter(|(_, _, value)| *value).count();
        let status = if super::verify_learned_agent_opinion_seal_v0(&replay.risk, &replay.risk_seal)
            .is_err()
        {
            CycleRiskOpinionWitnessStatusV0::OpinionSealInvalid
        } else if matching == 1 {
            CycleRiskOpinionWitnessStatusV0::VerifiedUniqueSource
        } else if matching == 0 {
            CycleRiskOpinionWitnessStatusV0::NoMatchingSource
        } else {
            CycleRiskOpinionWitnessStatusV0::MultipleMatchingSources
        };
        if matching == 0 {
            unmatched.push(replay.risk.opinion_id.clone());
        }
        if matching > 1 {
            multiple.push(replay.risk.opinion_id.clone());
        }
        for (identity, reconstructed, matched) in matches {
            let mut bytes = Vec::new();
            strv(&mut bytes, "risk-opinion-witness-v1");
            strv(&mut bytes, &replay.risk.opinion_digest);
            strv(&mut bytes, &identity.result_digest_v1);
            boolv(&mut bytes, matched);
            let reason = match status {
                CycleRiskOpinionWitnessStatusV0::VerifiedUniqueSource => {
                    "unique_reconstructed_result"
                }
                CycleRiskOpinionWitnessStatusV0::MultipleMatchingSources => {
                    "multiple_reconstructed_results"
                }
                CycleRiskOpinionWitnessStatusV0::NoMatchingSource => "no_reconstructed_result",
                _ => "witness_invalid",
            }
            .to_string();
            witnesses.push(CycleRiskOpinionProvenanceWitnessV0 {
                witness_version: "cycle-risk-opinion-provenance-witness-v1".into(),
                opinion_id: replay.risk.opinion_id.clone(),
                opinion_digest: replay.risk.opinion_digest.clone(),
                seal_digest: replay.risk_seal.seal_digest.clone(),
                candidate_result_digest_v1: identity.result_digest_v1.clone(),
                candidate_scope_digest_v1: identity.canonical_scope_digest_v1.clone(),
                reconstructed_opinion_digest: reconstructed.opinion_digest,
                reconstructed_matches_existing: matched,
                source_snapshot_matches: report.snapshot_id == identity.source_snapshot_id,
                pack_identity_matches: plan
                    .ranges
                    .iter()
                    .any(|range| range.expected_frozen_pack_digest == identity.frozen_pack_digest),
                objective_matches: replay.risk.objective == LearnedAgentObjectiveV0::DownsideRisk,
                agent_matches: replay.risk.agent_id == CYCLE_RISK_SHADOW_AGENT_ID_V0,
                witness_status: status,
                reason_codes: vec![reason],
                witness_digest_v1: stable_hash_string(&hex(&bytes)),
            });
        }
    }
    witnesses.sort_by(|left, right| left.witness_digest_v1.cmp(&right.witness_digest_v1));
    unmatched.sort();
    unmatched.dedup();
    multiple.sort();
    multiple.dedup();
    let status = if witnesses.is_empty() {
        CycleRiskProvenanceRegistryStatusV0::Empty
    } else if !multiple.is_empty() {
        CycleRiskProvenanceRegistryStatusV0::Ambiguous
    } else if unmatched.is_empty() {
        CycleRiskProvenanceRegistryStatusV0::FullyVerified
    } else {
        CycleRiskProvenanceRegistryStatusV0::PartiallyVerified
    };
    let digest = strings_digest_v1(
        "risk-provenance-registry-v1",
        &witnesses
            .iter()
            .map(|value| value.witness_digest_v1.clone())
            .collect::<Vec<_>>(),
    );
    CycleRiskOpinionProvenanceRegistryV0 {
        registry_version: "cycle-risk-opinion-provenance-registry-v1".into(),
        range_plan_digest: plan.plan_digest.clone(),
        result_identities: identities.to_vec(),
        witnesses,
        unmatched_opinion_ids: unmatched,
        multiply_matched_opinion_ids: multiple,
        status,
        registry_digest_v1: digest,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeAlignmentReportV1 {
    pub report_version: String,
    pub offline: bool,
    pub provider_calls: usize,
    pub transport_constructions: usize,
    pub network_consent_reads: usize,
    pub legacy: ScopeAlignmentReportV0,
    pub range_plan: CycleRiskHistoricalRangePlanV0,
    pub provenance: CycleRiskOpinionProvenanceRegistryV0,
    pub risk_anchor_scopes: Vec<AgentEffectiveAnchorScopeV1>,
    pub report_digest_v1: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpinionScopeAttestationStatusV1 {
    Verified,
    LegacyOpinionValidButSourceUnresolved,
    SourceWitnessMismatch,
    RawScopeMismatch,
    AnchorScopeInvalid,
    ForecastScopeInvalid,
    AgentMismatch,
    ObjectiveMismatch,
    TechnicalFailure,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentOpinionScopeAttestationV1 {
    pub attestation_version: String,
    pub opinion_id: String,
    pub opinion_digest: String,
    pub seal_digest: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub source_result_digest_v1: String,
    pub canonical_raw_scope_digest_v1: String,
    pub effective_anchor_scope_digest_v1: String,
    pub forecast_scope_digest_v1: String,
    pub provenance_witness_digest_v1: String,
    pub attestation_status: OpinionScopeAttestationStatusV1,
    pub reason_codes: Vec<String>,
    pub attestation_digest_v1: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearnedAgentRawScopeAlignmentV1 {
    ExactSameCanonicalRows,
    CertifiedEquivalentCanonicalRows,
    MomentumStrictSubsetOfRisk,
    RiskStrictSubsetOfMomentum,
    PartialOverlap,
    SameRangeDifferentRows,
    DifferentInformationCutoff,
    Disjoint,
    Ambiguous,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearnedAgentAnchorAlignmentV1 {
    ExactSameAnchors,
    SameRawScopeDifferentAnchors,
    MomentumAnchorsStrictSubset,
    RiskAnchorsStrictSubset,
    PartialAnchorOverlap,
    DisjointAnchors,
    Unknown,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectiveHorizonAlignmentV1 {
    SameHorizonDifferentObjective,
    DifferentHorizonCompatibleForRegimeSummary,
    DifferentHorizonNotComparable,
    MissingHorizonIdentity,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearnedAgentScopeComparabilityV1 {
    ExactDecisionScopeComparable,
    RegimeSummaryComparableWithCaveats,
    SharedLineageButNotComparable,
    PartialOverlapNotComparable,
    DifferentCutoffNotComparable,
    AmbiguousNotComparable,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeMappingRegistryStatusV1 {
    FullyMapped,
    FullyMappedWithCaveats,
    PartiallyMapped,
    ProvenanceVerifiedButScopesNotComparable,
    Ambiguous,
    Empty,
    Invalid,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalLearnedAgentScopePairV1 {
    pub pair_id: String,
    pub momentum_opinion_id: String,
    pub risk_opinion_id: String,
    pub momentum_attestation_digest_v1: String,
    pub risk_attestation_digest_v1: String,
    pub raw_scope_alignment: LearnedAgentRawScopeAlignmentV1,
    pub anchor_alignment: LearnedAgentAnchorAlignmentV1,
    pub horizon_alignment: ObjectiveHorizonAlignmentV1,
    pub comparability: LearnedAgentScopeComparabilityV1,
    pub caveat_codes: Vec<String>,
    pub pair_digest_v1: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentScopeMappingRegistryV1 {
    pub registry_version: String,
    pub momentum_attestations: Vec<LearnedAgentOpinionScopeAttestationV1>,
    pub risk_attestations: Vec<LearnedAgentOpinionScopeAttestationV1>,
    pub scope_pairs: Vec<CanonicalLearnedAgentScopePairV1>,
    pub unmatched_momentum_opinion_ids: Vec<String>,
    pub unmatched_risk_opinion_ids: Vec<String>,
    pub ambiguous_opinion_ids: Vec<String>,
    pub mapping_status: ScopeMappingRegistryStatusV1,
    pub legacy_v0_registry_digest: String,
    pub registry_digest_v1: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregateCompositionStatusV1 {
    FullyComposed,
    FullyComposedWithScopeCaveats,
    PartiallyMappedNotComposable,
    AmbiguousNotComposable,
    EmptyNotComposable,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregateRelationshipSummaryV1 {
    AllScopesBothAbstained,
    MostlyAbstained,
    TensionObserved,
    OrthogonalAcrossScopes,
    MixedRelationships,
    IncomparableEvidence,
    InsufficientMappedScopes,
    TechnicalFailure,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrossScopeShadowDeliberationAggregateV1 {
    pub aggregate_version: String,
    pub mapping_registry_digest_v1: String,
    pub included_deliberation_ids: Vec<String>,
    pub included_scope_pair_ids: Vec<String>,
    pub mapped_scope_count: usize,
    pub unmatched_scope_count: usize,
    pub both_abstained_count: usize,
    pub momentum_abstained_count: usize,
    pub risk_abstained_count: usize,
    pub compatible_count: usize,
    pub orthogonal_count: usize,
    pub tension_count: usize,
    pub incomparable_count: usize,
    pub relationship_summary: AggregateRelationshipSummaryV1,
    pub composition_status: AggregateCompositionStatusV1,
    pub no_winner_selected: bool,
    pub no_action_selected: bool,
    pub chair_observed: bool,
    pub reward_created: bool,
    pub penalty_created: bool,
    pub vote_created: bool,
    pub execution_created: bool,
    pub aggregate_digest_v1: String,
}
pub fn replay_btc_scope_alignment_v1(
    snapshot: &DataSnapshot,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<ScopeAlignmentReportV1, String> {
    let legacy = replay_btc_scope_alignment_v0(snapshot, campaign)?;
    let config = CycleRiskShadowConfigV0::default();
    let plan = cycle_risk_historical_range_plan_v0(snapshot, &config)?;
    let risk = super::run_cycle_risk_shadow_v0(snapshot, &config)
        .map_err(|_| "risk_provenance_replay_failed")?;
    let identities = risk
        .regimes
        .iter()
        .filter_map(|result| {
            let resolution = resolve_risk_range(&plan, result);
            risk_result_identity_v1(&risk, result, &config, &resolution)
        })
        .collect::<Vec<_>>();
    if identities.len() != risk.regimes.len() {
        return Err("risk_result_range_resolution_failed".into());
    }
    let replays = super::replay_btc_shadow_deliberations_v0(snapshot, campaign)?;
    let provenance = witness_registry_v1(&risk, &plan, &identities, &replays);
    let risk_anchor_scopes = plan
        .ranges
        .iter()
        .map(|candidate| risk_anchor_scope_v1(snapshot, candidate, &config))
        .collect::<Result<Vec<_>, _>>()?;
    let mut bytes = Vec::new();
    strv(&mut bytes, "learned-agent-scope-alignment-v1");
    strv(&mut bytes, &legacy.report_digest);
    strv(&mut bytes, &plan.plan_digest);
    strv(&mut bytes, &provenance.registry_digest_v1);
    strings(
        &mut bytes,
        &risk_anchor_scopes
            .iter()
            .map(|scope| scope.scope_digest_v1.clone())
            .collect::<Vec<_>>(),
    );
    Ok(ScopeAlignmentReportV1 {
        report_version: "learned-agent-scope-alignment-v1".into(),
        offline: true,
        provider_calls: 0,
        transport_constructions: 0,
        network_consent_reads: 0,
        legacy,
        range_plan: plan,
        provenance,
        risk_anchor_scopes,
        report_digest_v1: stable_hash_string(&hex(&bytes)),
    })
}

/// Immutable Phase-A policy.  This is deliberately separate from legacy V0
/// artifacts and is the only policy accepted by source-bound V1 constructors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBoundOpinionProtocolRegistrationV1 {
    pub registration_version: String,
    pub opinion_protocol_version: String,
    pub seal_protocol_version: String,
    pub deliberation_protocol_version: String,
    pub canonical_encoding_version: String,
    pub exact_source_result_required: bool,
    pub canonical_raw_scope_required: bool,
    pub effective_anchor_scope_required: bool,
    pub forecast_scope_required: bool,
    pub source_model_artifact_required: bool,
    pub caller_supplied_regime_alias_forbidden: bool,
    pub aggregate_report_as_primary_source_forbidden: bool,
    pub legacy_opinion_upgrade_forbidden: bool,
    pub legacy_digest_rewrite_forbidden: bool,
    pub retrospective_creation_mode_only: bool,
    pub opinion_seal_before_reveal_required: bool,
    pub exact_two_rounds_required: bool,
    pub advisory_only_required: bool,
    pub chair_eligibility_forbidden: bool,
    pub vote_eligibility_forbidden: bool,
    pub reward_eligibility_forbidden: bool,
    pub penalty_eligibility_forbidden: bool,
    pub promotion_eligibility_forbidden: bool,
    pub execution_eligibility_forbidden: bool,
    pub policy_digest_v1: String,
}
impl SourceBoundOpinionProtocolRegistrationV1 {
    pub fn pre_registered() -> Self {
        let mut value = Self {
            registration_version: "source-bound-opinion-registration-v1".into(),
            opinion_protocol_version: "learned-agent-opinion-v1".into(),
            seal_protocol_version: "learned-agent-opinion-seal-v1".into(),
            deliberation_protocol_version: "source-bound-shadow-deliberation-v1".into(),
            canonical_encoding_version: "canonical-semantic-encoding-v1".into(),
            exact_source_result_required: true,
            canonical_raw_scope_required: true,
            effective_anchor_scope_required: true,
            forecast_scope_required: true,
            source_model_artifact_required: true,
            caller_supplied_regime_alias_forbidden: true,
            aggregate_report_as_primary_source_forbidden: true,
            legacy_opinion_upgrade_forbidden: true,
            legacy_digest_rewrite_forbidden: true,
            retrospective_creation_mode_only: true,
            opinion_seal_before_reveal_required: true,
            exact_two_rounds_required: true,
            advisory_only_required: true,
            chair_eligibility_forbidden: true,
            vote_eligibility_forbidden: true,
            reward_eligibility_forbidden: true,
            penalty_eligibility_forbidden: true,
            promotion_eligibility_forbidden: true,
            execution_eligibility_forbidden: true,
            policy_digest_v1: String::new(),
        };
        value.policy_digest_v1 = source_bound_registration_digest_v1(&value);
        value
    }
    pub fn validate(&self) -> Result<(), String> {
        let flags = [
            self.exact_source_result_required,
            self.canonical_raw_scope_required,
            self.effective_anchor_scope_required,
            self.forecast_scope_required,
            self.source_model_artifact_required,
            self.caller_supplied_regime_alias_forbidden,
            self.aggregate_report_as_primary_source_forbidden,
            self.legacy_opinion_upgrade_forbidden,
            self.legacy_digest_rewrite_forbidden,
            self.retrospective_creation_mode_only,
            self.opinion_seal_before_reveal_required,
            self.exact_two_rounds_required,
            self.advisory_only_required,
            self.chair_eligibility_forbidden,
            self.vote_eligibility_forbidden,
            self.reward_eligibility_forbidden,
            self.penalty_eligibility_forbidden,
            self.promotion_eligibility_forbidden,
            self.execution_eligibility_forbidden,
        ];
        if flags.iter().any(|value| !value)
            || self.policy_digest_v1 != source_bound_registration_digest_v1(self)
        {
            return Err("invalid_source_bound_opinion_registration".into());
        }
        Ok(())
    }
}
fn source_bound_registration_digest_v1(value: &SourceBoundOpinionProtocolRegistrationV1) -> String {
    let mut bytes = Vec::new();
    strv(&mut bytes, "source-bound-opinion-registration-v1");
    for version in [
        &value.registration_version,
        &value.opinion_protocol_version,
        &value.seal_protocol_version,
        &value.deliberation_protocol_version,
        &value.canonical_encoding_version,
    ] {
        strv(&mut bytes, version);
    }
    for flag in [
        value.exact_source_result_required,
        value.canonical_raw_scope_required,
        value.effective_anchor_scope_required,
        value.forecast_scope_required,
        value.source_model_artifact_required,
        value.caller_supplied_regime_alias_forbidden,
        value.aggregate_report_as_primary_source_forbidden,
        value.legacy_opinion_upgrade_forbidden,
        value.legacy_digest_rewrite_forbidden,
        value.retrospective_creation_mode_only,
        value.opinion_seal_before_reveal_required,
        value.exact_two_rounds_required,
        value.advisory_only_required,
        value.chair_eligibility_forbidden,
        value.vote_eligibility_forbidden,
        value.reward_eligibility_forbidden,
        value.penalty_eligibility_forbidden,
        value.promotion_eligibility_forbidden,
        value.execution_eligibility_forbidden,
    ] {
        boolv(&mut bytes, flag);
    }
    stable_hash_string(&hex(&bytes))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearnedAgentOpinionCreationModeV1 {
    HistoricalRetrospectiveSourceBoundReplay,
    RetrospectiveJointScopeDevelopmentReplay,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceResultKindV1 {
    MomentumHistoricalRegimeResult,
    CycleRiskHistoricalRegimeResult,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentSourceResultReferenceV1 {
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub source_snapshot_id: String,
    pub source_snapshot_digest: String,
    pub source_result_kind: SourceResultKindV1,
    pub source_result_digest_v1: String,
    pub source_checkpoint_digest_v1: String,
    pub source_frozen_pack_digest: String,
    pub source_model_version_id: Option<String>,
    pub source_model_artifact_digest: String,
    pub canonical_raw_scope_digest_v1: String,
    /// Ordered, canonical row identities retained with the reference so a
    /// later retrospective mapper can prove overlap instead of guessing from
    /// a scope name or digest prefix.
    pub canonical_raw_row_identity_digests_v1: Vec<String>,
    pub information_cutoff_timestamp: u64,
    pub effective_anchor_scope_digest_v1: String,
    pub effective_anchor_digests_v1: Vec<String>,
    pub forecast_scope_digest_v1: String,
    pub reference_digest_v1: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceResultMembershipProofV1 {
    pub result_digest_v1: String,
    pub parent_report_digest: String,
    pub immutable_member: bool,
    pub snapshot_matches: bool,
    pub pack_matches: bool,
    pub scope_matches: bool,
    pub anchors_match: bool,
    pub objective_matches: bool,
    pub agent_matches: bool,
    pub all_invariants_pass: bool,
    pub proof_digest_v1: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpinionAuthorityV1 {
    pub advisory_only: bool,
    pub eligible_to_vote: bool,
    pub eligible_to_reach_chair: bool,
    pub eligible_for_reward: bool,
    pub eligible_for_penalty: bool,
    pub eligible_for_speaking_right_change: bool,
    pub eligible_for_promotion: bool,
    pub eligible_to_execute: bool,
}
impl OpinionAuthorityV1 {
    pub fn historical_advisory_only() -> Self {
        Self {
            advisory_only: true,
            eligible_to_vote: false,
            eligible_to_reach_chair: false,
            eligible_for_reward: false,
            eligible_for_penalty: false,
            eligible_for_speaking_right_change: false,
            eligible_for_promotion: false,
            eligible_to_execute: false,
        }
    }
}
fn authority_digest_v1(value: &OpinionAuthorityV1) -> String {
    let mut bytes = Vec::new();
    for flag in [
        value.advisory_only,
        value.eligible_to_vote,
        value.eligible_to_reach_chair,
        value.eligible_for_reward,
        value.eligible_for_penalty,
        value.eligible_for_speaking_right_change,
        value.eligible_for_promotion,
        value.eligible_to_execute,
    ] {
        boolv(&mut bytes, flag);
    }
    stable_hash_string(&hex(&bytes))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentOpinionSealV1 {
    pub seal_version: String,
    pub opinion_id: String,
    pub opinion_digest_v1: String,
    pub source_result_digest_v1: String,
    pub canonical_raw_scope_digest_v1: String,
    pub effective_anchor_scope_digest_v1: String,
    pub forecast_scope_digest_v1: String,
    pub protocol_registration_digest_v1: String,
    pub authority_digest_v1: String,
    pub sealed_before_cross_agent_reveal: bool,
    pub seal_digest_v1: String,
}
pub fn source_bound_seal_v1(
    opinion_id: &str,
    opinion_digest: &str,
    source: &LearnedAgentSourceResultReferenceV1,
    registration: &SourceBoundOpinionProtocolRegistrationV1,
    authority: &OpinionAuthorityV1,
) -> Result<LearnedAgentOpinionSealV1, String> {
    registration.validate()?;
    if authority != &OpinionAuthorityV1::historical_advisory_only() {
        return Err("source_bound_authority_violation".into());
    }
    let mut value = LearnedAgentOpinionSealV1 {
        seal_version: registration.seal_protocol_version.clone(),
        opinion_id: opinion_id.into(),
        opinion_digest_v1: opinion_digest.into(),
        source_result_digest_v1: source.source_result_digest_v1.clone(),
        canonical_raw_scope_digest_v1: source.canonical_raw_scope_digest_v1.clone(),
        effective_anchor_scope_digest_v1: source.effective_anchor_scope_digest_v1.clone(),
        forecast_scope_digest_v1: source.forecast_scope_digest_v1.clone(),
        protocol_registration_digest_v1: registration.policy_digest_v1.clone(),
        authority_digest_v1: authority_digest_v1(authority),
        sealed_before_cross_agent_reveal: true,
        seal_digest_v1: String::new(),
    };
    let mut bytes = Vec::new();
    for item in [
        &value.seal_version,
        &value.opinion_id,
        &value.opinion_digest_v1,
        &value.source_result_digest_v1,
        &value.canonical_raw_scope_digest_v1,
        &value.effective_anchor_scope_digest_v1,
        &value.forecast_scope_digest_v1,
        &value.protocol_registration_digest_v1,
        &value.authority_digest_v1,
    ] {
        strv(&mut bytes, item);
    }
    boolv(&mut bytes, value.sealed_before_cross_agent_reveal);
    value.seal_digest_v1 = stable_hash_string(&hex(&bytes));
    Ok(value)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpinionTemporalScopeV1 {
    pub source_information_cutoff_timestamp: u64,
    pub forecast_horizon_digest_v1: String,
    pub prospective: bool,
    pub contemporaneous_claim: bool,
    pub scope_digest_v1: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentOpinionEnvelopeV1 {
    pub protocol_version: String,
    pub opinion_id: String,
    pub creation_mode: LearnedAgentOpinionCreationModeV1,
    pub protocol_registration_digest_v1: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub doctrine_id: String,
    pub source_result: LearnedAgentSourceResultReferenceV1,
    pub source_membership_proof_digest_v1: String,
    pub temporal_scope: OpinionTemporalScopeV1,
    pub reason_codes: Vec<String>,
    pub authority: OpinionAuthorityV1,
    pub sealed: bool,
    pub opinion_digest_v1: String,
}
fn objective_tag_v1(value: LearnedAgentObjectiveV0) -> u8 {
    match value {
        LearnedAgentObjectiveV0::DirectionalMomentum => 1,
        LearnedAgentObjectiveV0::DownsideRisk => 2,
    }
}
fn source_reference_digest_v1(value: &LearnedAgentSourceResultReferenceV1) -> String {
    let mut bytes = Vec::new();
    strv(&mut bytes, "learned-agent-source-result-reference-v1");
    strv(&mut bytes, &value.agent_id);
    tag(&mut bytes, objective_tag_v1(value.objective));
    for text in [
        &value.source_snapshot_id,
        &value.source_snapshot_digest,
        &value.source_result_digest_v1,
        &value.source_checkpoint_digest_v1,
        &value.source_frozen_pack_digest,
        &value.source_model_artifact_digest,
        &value.canonical_raw_scope_digest_v1,
        &value.effective_anchor_scope_digest_v1,
        &value.forecast_scope_digest_v1,
    ] {
        strv(&mut bytes, text);
    }
    strings(&mut bytes, &value.canonical_raw_row_identity_digests_v1);
    u64v(&mut bytes, value.information_cutoff_timestamp);
    strings(&mut bytes, &value.effective_anchor_digests_v1);
    tag(
        &mut bytes,
        match value.source_result_kind {
            SourceResultKindV1::MomentumHistoricalRegimeResult => 1,
            SourceResultKindV1::CycleRiskHistoricalRegimeResult => 2,
        },
    );
    opt_strv(&mut bytes, value.source_model_version_id.as_deref());
    stable_hash_string(&hex(&bytes))
}
fn opinion_envelope_digest_v1(value: &LearnedAgentOpinionEnvelopeV1) -> String {
    let mut bytes = Vec::new();
    for text in [
        &value.protocol_version,
        &value.opinion_id,
        &value.protocol_registration_digest_v1,
        &value.agent_id,
        &value.doctrine_id,
        &value.source_result.reference_digest_v1,
        &value.source_membership_proof_digest_v1,
        &value.temporal_scope.scope_digest_v1,
    ] {
        strv(&mut bytes, text);
    }
    tag(&mut bytes, objective_tag_v1(value.objective));
    strings(&mut bytes, &value.reason_codes);
    strv(&mut bytes, &authority_digest_v1(&value.authority));
    stable_hash_string(&hex(&bytes))
}
pub fn create_source_bound_opinion_v1(
    source: LearnedAgentSourceResultReferenceV1,
    membership: &SourceResultMembershipProofV1,
    cutoff: u64,
    doctrine_id: &str,
    registration: &SourceBoundOpinionProtocolRegistrationV1,
) -> Result<LearnedAgentOpinionEnvelopeV1, String> {
    registration.validate()?;
    if !membership.all_invariants_pass
        || membership.result_digest_v1 != source.source_result_digest_v1
        || source.canonical_raw_row_identity_digests_v1.is_empty()
        || source.effective_anchor_digests_v1.is_empty()
        || source.reference_digest_v1 != source_reference_digest_v1(&source)
    {
        return Err("source_bound_membership_invalid".into());
    }
    let mut temporal = OpinionTemporalScopeV1 {
        source_information_cutoff_timestamp: cutoff,
        forecast_horizon_digest_v1: source.forecast_scope_digest_v1.clone(),
        prospective: false,
        contemporaneous_claim: false,
        scope_digest_v1: String::new(),
    };
    let mut temporal_bytes = Vec::new();
    u64v(&mut temporal_bytes, cutoff);
    strv(&mut temporal_bytes, &temporal.forecast_horizon_digest_v1);
    boolv(&mut temporal_bytes, false);
    boolv(&mut temporal_bytes, false);
    temporal.scope_digest_v1 = stable_hash_string(&hex(&temporal_bytes));
    let authority = OpinionAuthorityV1::historical_advisory_only();
    let mut id_bytes = Vec::new();
    strv(&mut id_bytes, "source-bound-opinion-v1");
    strv(&mut id_bytes, &registration.policy_digest_v1);
    strv(&mut id_bytes, &source.agent_id);
    tag(&mut id_bytes, objective_tag_v1(source.objective));
    for value in [
        &source.source_result_digest_v1,
        &source.canonical_raw_scope_digest_v1,
        &source.effective_anchor_scope_digest_v1,
        &source.forecast_scope_digest_v1,
        &source.source_model_artifact_digest,
    ] {
        strv(&mut id_bytes, value);
    }
    let opinion_id = format!(
        "source-bound-opinion-{}",
        stable_hash_string(&hex(&id_bytes))
    );
    let mut value = LearnedAgentOpinionEnvelopeV1 {
        protocol_version: registration.opinion_protocol_version.clone(),
        opinion_id,
        creation_mode: LearnedAgentOpinionCreationModeV1::HistoricalRetrospectiveSourceBoundReplay,
        protocol_registration_digest_v1: registration.policy_digest_v1.clone(),
        agent_id: source.agent_id.clone(),
        objective: source.objective,
        doctrine_id: doctrine_id.into(),
        source_result: source,
        source_membership_proof_digest_v1: membership.proof_digest_v1.clone(),
        temporal_scope: temporal,
        reason_codes: vec!["historical_retrospective_source_bound_abstention".into()],
        authority,
        sealed: false,
        opinion_digest_v1: String::new(),
    };
    value.opinion_digest_v1 = opinion_envelope_digest_v1(&value);
    Ok(value)
}

pub fn create_joint_scope_source_bound_opinion_v1(
    source: LearnedAgentSourceResultReferenceV1,
    membership: &SourceResultMembershipProofV1,
    cutoff: u64,
    joint_scope_id: &str,
    reason_code: &str,
    registration: &SourceBoundOpinionProtocolRegistrationV1,
) -> Result<LearnedAgentOpinionEnvelopeV1, String> {
    let mut opinion = create_source_bound_opinion_v1(
        source,
        membership,
        cutoff,
        "retrospective-joint-scope-development",
        registration,
    )?;
    opinion.creation_mode =
        LearnedAgentOpinionCreationModeV1::RetrospectiveJointScopeDevelopmentReplay;
    opinion.reason_codes = vec![reason_code.into()];
    opinion.opinion_id = format!(
        "joint-source-bound-opinion-{}",
        stable_hash_string(&format!(
            "{}:{}:{}",
            joint_scope_id,
            opinion.source_result.reference_digest_v1,
            registration.policy_digest_v1
        ))
    );
    opinion.opinion_digest_v1 = opinion_envelope_digest_v1(&opinion);
    Ok(opinion)
}

pub fn replay_source_bound_cycle_risk_opinions_v1(
    snapshot: &DataSnapshot,
) -> Result<Vec<(LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1)>, String> {
    let registration = SourceBoundOpinionProtocolRegistrationV1::pre_registered();
    registration.validate()?;
    let config = CycleRiskShadowConfigV0::default();
    let plan = cycle_risk_historical_range_plan_v0(snapshot, &config)?;
    let report = super::run_cycle_risk_shadow_v0(snapshot, &config)
        .map_err(|_| "source_bound_risk_report_failed")?;
    let mut output = Vec::new();
    for result in &report.regimes {
        let resolution = resolve_risk_range(&plan, result);
        let identity = risk_result_identity_v1(&report, result, &config, &resolution)
            .ok_or("source_bound_risk_identity_failed")?;
        let candidate = plan
            .ranges
            .iter()
            .find(|value| value.expected_frozen_pack_digest == result.frozen_pack_digest)
            .ok_or("source_bound_risk_range_missing")?;
        let anchors = risk_anchor_scope_v1(snapshot, candidate, &config)?;
        let mut source = LearnedAgentSourceResultReferenceV1 {
            agent_id: report.agent_id.clone(),
            objective: LearnedAgentObjectiveV0::DownsideRisk,
            source_snapshot_id: report.snapshot_id.clone(),
            source_snapshot_digest: report.snapshot_digest.clone(),
            source_result_kind: SourceResultKindV1::CycleRiskHistoricalRegimeResult,
            source_result_digest_v1: identity.result_digest_v1.clone(),
            source_checkpoint_digest_v1: identity.checkpoint_identity_digest.clone(),
            source_frozen_pack_digest: result.frozen_pack_digest.clone(),
            source_model_version_id: result.checkpoint.accepted_model_version.clone(),
            source_model_artifact_digest: result
                .checkpoint
                .accepted_model_version
                .clone()
                .unwrap_or_else(|| result.frozen_pack_digest.clone()),
            canonical_raw_scope_digest_v1: identity.canonical_scope_digest_v1.clone(),
            canonical_raw_row_identity_digests_v1: canonical_raw_scope_v1(
                snapshot,
                candidate.start_row_index,
                candidate.end_row_index_exclusive,
                &risk_config_digest_v1(&config),
            )?
            .row_identity_digests,
            information_cutoff_timestamp: snapshot.normalized_dataset.rows
                [candidate.end_row_index_exclusive - 1]
                .timestamp_ms,
            effective_anchor_scope_digest_v1: anchors.scope_digest_v1,
            effective_anchor_digests_v1: anchors.all_anchor_digests,
            forecast_scope_digest_v1: strings_digest_v1(
                "risk-forecast-scope-v1",
                &[config.label.horizon_rows.to_string(), config.label.digest()],
            ),
            reference_digest_v1: String::new(),
        };
        source.reference_digest_v1 = source_reference_digest_v1(&source);
        let mut proof = SourceResultMembershipProofV1 {
            result_digest_v1: source.source_result_digest_v1.clone(),
            parent_report_digest: report.ledger_digest.clone(),
            immutable_member: report.regimes.iter().any(|item| item == result),
            snapshot_matches: result.source_snapshot_id == report.snapshot_id,
            pack_matches: resolution.status
                == CycleRiskRangeResolutionStatusV0::VerifiedUniqueMatch,
            scope_matches: source.canonical_raw_scope_digest_v1
                == identity.canonical_scope_digest_v1,
            anchors_match: true,
            objective_matches: source.objective == LearnedAgentObjectiveV0::DownsideRisk,
            agent_matches: source.agent_id == CYCLE_RISK_SHADOW_AGENT_ID_V0,
            all_invariants_pass: false,
            proof_digest_v1: String::new(),
        };
        proof.all_invariants_pass = proof.immutable_member
            && proof.snapshot_matches
            && proof.pack_matches
            && proof.scope_matches
            && proof.anchors_match
            && proof.objective_matches
            && proof.agent_matches;
        let mut proof_bytes = Vec::new();
        for text in [&proof.result_digest_v1, &proof.parent_report_digest] {
            strv(&mut proof_bytes, text);
        }
        for flag in [
            proof.immutable_member,
            proof.snapshot_matches,
            proof.pack_matches,
            proof.scope_matches,
            proof.anchors_match,
            proof.objective_matches,
            proof.agent_matches,
            proof.all_invariants_pass,
        ] {
            boolv(&mut proof_bytes, flag);
        }
        proof.proof_digest_v1 = stable_hash_string(&hex(&proof_bytes));
        let mut opinion = create_source_bound_opinion_v1(
            source,
            &proof,
            snapshot.normalized_dataset.rows[candidate.end_row_index_exclusive - 1].timestamp_ms,
            "cycle-risk-historical-shadow",
            &registration,
        )?;
        let seal = source_bound_seal_v1(
            &opinion.opinion_id,
            &opinion.opinion_digest_v1,
            &opinion.source_result,
            &registration,
            &opinion.authority,
        )?;
        opinion.sealed = true;
        output.push((opinion, seal));
    }
    Ok(output)
}

pub fn replay_source_bound_momentum_opinions_v1(
    snapshot: &DataSnapshot,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<Vec<(LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1)>, String> {
    let registration = SourceBoundOpinionProtocolRegistrationV1::pre_registered();
    registration.validate()?;
    let sufficiency = assess_momentum_campaign_sufficiency_v0(snapshot.row_count, campaign)
        .map_err(|_| "momentum_sufficiency_invalid")?;
    let segmentation = segment_btc_historical_regimes_v0(
        snapshot,
        &BtcHistoricalRegimeConfigV0 {
            minimum_regimes: 2,
            regime_rows: sufficiency.required_minimum_rows,
            inter_regime_gap_rows: campaign.purge_gap_rows,
            minimum_campaign_windows_per_regime: campaign.minimum_evaluated_windows,
            segmentation_policy: TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
        },
    )
    .map_err(|_| "momentum_segmentation_invalid")?;
    let packs = freeze_btc_historical_regime_packs_v0(
        snapshot,
        &segmentation,
        &HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| "momentum_pack_invalid")?;
    let encoder = frozen_mamba3_encoder_from_seed_v0(
        &campaign.feature_config,
        campaign.campaign_seed,
        campaign.backend_preference,
        campaign.fallback_policy,
    )
    .map_err(|_| "momentum_encoder_invalid")?;
    let raw = run_btc_historical_regime_campaigns_v0(&packs, campaign, &encoder)
        .map_err(|_| "momentum_campaign_invalid")?;
    let mut output = Vec::new();
    for (rank, (regime, pack)) in packs.iter().enumerate() {
        let result = raw
            .iter()
            .find(|value| value.regime_id == regime.regime_id)
            .ok_or("momentum_result_missing")?;
        let closed = close_btc_temporal_regime_result_v0(
            result,
            BtcTemporalRegimeRefV0 {
                regime_id: regime.regime_id.clone(),
                chronological_rank: rank,
                row_count: regime.row_count,
                range_digest: stable_hash_string(&format!(
                    "{}:{}:{}",
                    regime.start_timestamp_ms, regime.end_timestamp_ms, regime.row_count
                )),
                pack_digest: pack.digest.clone(),
            },
        );
        let anchors = momentum_anchor_scope_v1(
            snapshot,
            regime.start_row_index,
            regime.end_row_index_exclusive,
            campaign,
        )?;
        let scope = canonical_raw_scope_v1(
            snapshot,
            regime.start_row_index,
            regime.end_row_index_exclusive,
            &segmentation.segmentation_config_digest,
        )?;
        let checkpoint = strings_digest_v1(
            "momentum-closed-checkpoint-v1",
            &[
                closed.report_digest.clone(),
                closed.execution_trace.trace_digest.clone(),
                closed.accepted_predictive_versions.to_string(),
            ],
        );
        let result_digest = strings_digest_v1(
            "momentum-regime-result-v1",
            &[
                snapshot.snapshot_id.clone(),
                snapshot.content_digest.clone(),
                pack.digest.clone(),
                scope.scope_digest_v1.clone(),
                anchors.scope_digest_v1.clone(),
                campaign.digest(),
                checkpoint.clone(),
                closed.report_digest.clone(),
            ],
        );
        let mut source = LearnedAgentSourceResultReferenceV1 {
            agent_id: campaign.agent_id.clone(),
            objective: LearnedAgentObjectiveV0::DirectionalMomentum,
            source_snapshot_id: snapshot.snapshot_id.clone(),
            source_snapshot_digest: snapshot.content_digest.clone(),
            source_result_kind: SourceResultKindV1::MomentumHistoricalRegimeResult,
            source_result_digest_v1: result_digest,
            source_checkpoint_digest_v1: checkpoint,
            source_frozen_pack_digest: pack.digest.clone(),
            source_model_version_id: None,
            source_model_artifact_digest: result.encoder_parameter_digest.clone(),
            canonical_raw_scope_digest_v1: scope.scope_digest_v1,
            canonical_raw_row_identity_digests_v1: scope.row_identity_digests,
            information_cutoff_timestamp: scope.information_cutoff_timestamp,
            effective_anchor_scope_digest_v1: anchors.scope_digest_v1,
            effective_anchor_digests_v1: anchors.all_anchor_digests,
            forecast_scope_digest_v1: strings_digest_v1(
                "momentum-forecast-scope-v1",
                &[
                    campaign.sequence_config.prediction_horizon.to_string(),
                    campaign
                        .sequence_config
                        .label_dead_zone
                        .to_bits()
                        .to_string(),
                ],
            ),
            reference_digest_v1: String::new(),
        };
        source.reference_digest_v1 = source_reference_digest_v1(&source);
        let mut proof = SourceResultMembershipProofV1 {
            result_digest_v1: source.source_result_digest_v1.clone(),
            parent_report_digest: closed.report_digest.clone(),
            immutable_member: true,
            snapshot_matches: true,
            pack_matches: closed.regime.pack_digest == source.source_frozen_pack_digest,
            scope_matches: true,
            anchors_match: true,
            objective_matches: true,
            agent_matches: true,
            all_invariants_pass: true,
            proof_digest_v1: String::new(),
        };
        proof.proof_digest_v1 = strings_digest_v1(
            "momentum-membership-proof-v1",
            &[
                proof.result_digest_v1.clone(),
                proof.parent_report_digest.clone(),
                source.reference_digest_v1.clone(),
            ],
        );
        let mut opinion = create_source_bound_opinion_v1(
            source,
            &proof,
            regime.end_timestamp_ms,
            "momentum-historical-shadow",
            &registration,
        )?;
        let seal = source_bound_seal_v1(
            &opinion.opinion_id,
            &opinion.opinion_digest_v1,
            &opinion.source_result,
            &registration,
            &opinion.authority,
        )?;
        opinion.sealed = true;
        output.push((opinion, seal));
    }
    Ok(output)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBoundMappingStatusV1 {
    FullyMapped,
    FullyMappedWithCaveats,
    SourceBoundButScopesNotComparable,
    PartiallyMapped,
    Empty,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBoundRawScopeAlignmentV1 {
    ExactSameCanonicalRows,
    PartialOverlap,
    DifferentInformationCutoff,
    Disjoint,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBoundAnchorAlignmentV1 {
    ExactSameAnchors,
    SameRawScopeDifferentAnchors,
    PartialOverlapAnchors,
    DisjointAnchors,
    Invalid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceBoundScopeComparabilityV1 {
    ExactDecisionScopeComparable,
    RegimeSummaryComparableWithCaveats,
    SourceBoundButScopesNotComparable,
    Invalid,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBoundOpinionScopePairV1 {
    pub pair_id: String,
    pub momentum_opinion_id: String,
    pub risk_opinion_id: String,
    pub raw_scope_alignment: SourceBoundRawScopeAlignmentV1,
    pub anchor_alignment: SourceBoundAnchorAlignmentV1,
    pub comparability: SourceBoundScopeComparabilityV1,
    pub pair_digest_v1: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBoundOpinionScopeMappingRegistryV1 {
    pub registry_version: String,
    pub protocol_registration_digest_v1: String,
    pub momentum_opinion_ids: Vec<String>,
    pub risk_opinion_ids: Vec<String>,
    pub scope_pairs: Vec<SourceBoundOpinionScopePairV1>,
    pub unmatched_momentum_opinion_ids: Vec<String>,
    pub unmatched_risk_opinion_ids: Vec<String>,
    pub non_comparable_pair_ids: Vec<String>,
    pub mapping_status: SourceBoundMappingStatusV1,
    pub registry_digest_v1: String,
}
pub fn map_source_bound_opinions_v1(
    momentum: &[(LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1)],
    risk: &[(LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1)],
) -> Result<SourceBoundOpinionScopeMappingRegistryV1, String> {
    let registration = SourceBoundOpinionProtocolRegistrationV1::pre_registered();
    registration.validate()?;
    let mut pairs = Vec::new();
    let mut used_risk = Vec::new();
    let mut unmatched_momentum = Vec::new();
    let mut non_comparable = Vec::new();
    for (momentum_opinion, momentum_seal) in momentum {
        if !momentum_opinion.sealed || !momentum_seal.sealed_before_cross_agent_reveal {
            return Err("unsealed_momentum_source_bound_opinion".into());
        }
        if momentum_opinion
            .source_result
            .canonical_raw_row_identity_digests_v1
            .is_empty()
            || momentum_opinion
                .source_result
                .effective_anchor_digests_v1
                .is_empty()
        {
            return Err("momentum_source_bound_scope_identity_missing".into());
        }
        let mut candidates = risk
            .iter()
            .filter_map(|(risk_opinion, risk_seal)| {
                if !risk_opinion.sealed
                    || !risk_seal.sealed_before_cross_agent_reveal
                    || used_risk.contains(&risk_opinion.opinion_id)
                    || risk_opinion.source_result.source_snapshot_digest
                        != momentum_opinion.source_result.source_snapshot_digest
                    || risk_opinion
                        .source_result
                        .canonical_raw_row_identity_digests_v1
                        .is_empty()
                    || risk_opinion
                        .source_result
                        .effective_anchor_digests_v1
                        .is_empty()
                {
                    return None;
                }
                let overlap = momentum_opinion
                    .source_result
                    .canonical_raw_row_identity_digests_v1
                    .iter()
                    .filter(|row| {
                        risk_opinion
                            .source_result
                            .canonical_raw_row_identity_digests_v1
                            .contains(*row)
                    })
                    .count();
                (overlap > 0).then_some((overlap, risk_opinion))
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| left.1.opinion_id.cmp(&right.1.opinion_id))
        });
        let Some((best_overlap, risk_opinion)) = candidates.first() else {
            unmatched_momentum.push(momentum_opinion.opinion_id.clone());
            continue;
        };
        if candidates
            .iter()
            .filter(|(overlap, _)| overlap == best_overlap)
            .count()
            != 1
        {
            unmatched_momentum.push(momentum_opinion.opinion_id.clone());
            continue;
        }
        used_risk.push(risk_opinion.opinion_id.clone());
        let raw_rows_equal = momentum_opinion
            .source_result
            .canonical_raw_row_identity_digests_v1
            == risk_opinion
                .source_result
                .canonical_raw_row_identity_digests_v1;
        let raw = if momentum_opinion.source_result.information_cutoff_timestamp
            != risk_opinion.source_result.information_cutoff_timestamp
        {
            SourceBoundRawScopeAlignmentV1::DifferentInformationCutoff
        } else if raw_rows_equal {
            SourceBoundRawScopeAlignmentV1::ExactSameCanonicalRows
        } else {
            SourceBoundRawScopeAlignmentV1::PartialOverlap
        };
        let anchors_equal = momentum_opinion.source_result.effective_anchor_digests_v1
            == risk_opinion.source_result.effective_anchor_digests_v1;
        let anchor_overlap = momentum_opinion
            .source_result
            .effective_anchor_digests_v1
            .iter()
            .filter(|anchor| {
                risk_opinion
                    .source_result
                    .effective_anchor_digests_v1
                    .contains(*anchor)
            })
            .count();
        let anchor = if anchors_equal {
            SourceBoundAnchorAlignmentV1::ExactSameAnchors
        } else if anchor_overlap > 0 {
            SourceBoundAnchorAlignmentV1::PartialOverlapAnchors
        } else {
            SourceBoundAnchorAlignmentV1::DisjointAnchors
        };
        let comparability = if raw == SourceBoundRawScopeAlignmentV1::ExactSameCanonicalRows
            && anchors_equal
            && momentum_opinion.source_result.forecast_scope_digest_v1
                == risk_opinion.source_result.forecast_scope_digest_v1
        {
            SourceBoundScopeComparabilityV1::ExactDecisionScopeComparable
        } else if raw == SourceBoundRawScopeAlignmentV1::ExactSameCanonicalRows {
            SourceBoundScopeComparabilityV1::RegimeSummaryComparableWithCaveats
        } else {
            SourceBoundScopeComparabilityV1::SourceBoundButScopesNotComparable
        };
        let mut bytes = Vec::new();
        strv(&mut bytes, &momentum_opinion.opinion_id);
        strv(&mut bytes, &risk_opinion.opinion_id);
        tag(
            &mut bytes,
            match raw {
                SourceBoundRawScopeAlignmentV1::ExactSameCanonicalRows => 1,
                SourceBoundRawScopeAlignmentV1::PartialOverlap => 2,
                SourceBoundRawScopeAlignmentV1::DifferentInformationCutoff => 3,
                SourceBoundRawScopeAlignmentV1::Disjoint => 4,
                SourceBoundRawScopeAlignmentV1::Invalid => 5,
            },
        );
        tag(
            &mut bytes,
            match anchor {
                SourceBoundAnchorAlignmentV1::ExactSameAnchors => 1,
                SourceBoundAnchorAlignmentV1::SameRawScopeDifferentAnchors => 2,
                SourceBoundAnchorAlignmentV1::PartialOverlapAnchors => 3,
                SourceBoundAnchorAlignmentV1::DisjointAnchors => 4,
                SourceBoundAnchorAlignmentV1::Invalid => 5,
            },
        );
        let digest = stable_hash_string(&hex(&bytes));
        if comparability != SourceBoundScopeComparabilityV1::ExactDecisionScopeComparable {
            non_comparable.push(format!("pair-{digest}"));
        }
        pairs.push(SourceBoundOpinionScopePairV1 {
            pair_id: format!("pair-{digest}"),
            momentum_opinion_id: momentum_opinion.opinion_id.clone(),
            risk_opinion_id: risk_opinion.opinion_id.clone(),
            raw_scope_alignment: raw,
            anchor_alignment: anchor,
            comparability,
            pair_digest_v1: digest,
        });
    }
    let mut unmatched_risk = risk
        .iter()
        .filter(|(opinion, _)| !used_risk.contains(&opinion.opinion_id))
        .map(|(opinion, _)| opinion.opinion_id.clone())
        .collect::<Vec<_>>();
    unmatched_risk.sort();
    let status = if momentum.is_empty() || risk.is_empty() {
        SourceBoundMappingStatusV1::Empty
    } else if pairs.is_empty() {
        SourceBoundMappingStatusV1::SourceBoundButScopesNotComparable
    } else if !unmatched_momentum.is_empty() || !unmatched_risk.is_empty() {
        SourceBoundMappingStatusV1::PartiallyMapped
    } else if non_comparable.len() == pairs.len() {
        SourceBoundMappingStatusV1::SourceBoundButScopesNotComparable
    } else if non_comparable.is_empty() {
        SourceBoundMappingStatusV1::FullyMapped
    } else {
        SourceBoundMappingStatusV1::FullyMappedWithCaveats
    };
    let mut bytes = Vec::new();
    strings(
        &mut bytes,
        &pairs
            .iter()
            .map(|pair| pair.pair_digest_v1.clone())
            .collect::<Vec<_>>(),
    );
    strings(&mut bytes, &unmatched_momentum);
    strings(&mut bytes, &unmatched_risk);
    Ok(SourceBoundOpinionScopeMappingRegistryV1 {
        registry_version: "source-bound-opinion-scope-mapping-v1".into(),
        protocol_registration_digest_v1: registration.policy_digest_v1,
        momentum_opinion_ids: momentum
            .iter()
            .map(|(opinion, _)| opinion.opinion_id.clone())
            .collect(),
        risk_opinion_ids: risk
            .iter()
            .map(|(opinion, _)| opinion.opinion_id.clone())
            .collect(),
        scope_pairs: pairs,
        unmatched_momentum_opinion_ids: unmatched_momentum,
        unmatched_risk_opinion_ids: unmatched_risk,
        non_comparable_pair_ids: non_comparable,
        mapping_status: status,
        registry_digest_v1: stable_hash_string(&hex(&bytes)),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBoundShadowDeliberationV1 {
    pub deliberation_version: String,
    pub creation_mode: LearnedAgentOpinionCreationModeV1,
    pub protocol_registration_digest_v1: String,
    pub momentum_opinion_id: String,
    pub risk_opinion_id: String,
    pub momentum_seal_digest_v1: String,
    pub risk_seal_digest_v1: String,
    pub relationship: AgentOpinionRelationshipV0,
    pub relationship_digest_v1: String,
    pub round_count: usize,
    pub retrospective_only: bool,
    pub contemporaneous_claim: bool,
    pub chair_observed: bool,
    pub chair_decision_created: bool,
    pub reward_created: bool,
    pub penalty_created: bool,
    pub speaking_right_changed: bool,
    pub vote_created: bool,
    pub execution_created: bool,
    pub transcript_digest_v1: String,
}
pub fn create_source_bound_deliberation_v1(
    pair: &SourceBoundOpinionScopePairV1,
    momentum: &(LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1),
    risk: &(LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1),
) -> Result<SourceBoundShadowDeliberationV1, String> {
    if pair.comparability == SourceBoundScopeComparabilityV1::SourceBoundButScopesNotComparable
        || !momentum.0.sealed
        || !risk.0.sealed
        || momentum.0.opinion_id != pair.momentum_opinion_id
        || risk.0.opinion_id != pair.risk_opinion_id
    {
        return Err("source_bound_deliberation_ineligible".into());
    }
    let registration = SourceBoundOpinionProtocolRegistrationV1::pre_registered();
    registration.validate()?;
    // V1 opinions are historical abstentions by construction.  Record the
    // actual relationship explicitly instead of reducing it to an opaque
    // digest or pretending there is a directional winner.
    let relationship = AgentOpinionRelationshipV0::BothAbstained;
    let mut relationship_bytes = Vec::new();
    strv(&mut relationship_bytes, &pair.pair_digest_v1);
    tag(
        &mut relationship_bytes,
        if pair.comparability == SourceBoundScopeComparabilityV1::ExactDecisionScopeComparable {
            1
        } else {
            2
        },
    );
    tag(&mut relationship_bytes, 7);
    let relationship_digest = stable_hash_string(&hex(&relationship_bytes));
    let mut value = SourceBoundShadowDeliberationV1 {
        deliberation_version: registration.deliberation_protocol_version.clone(),
        creation_mode: LearnedAgentOpinionCreationModeV1::HistoricalRetrospectiveSourceBoundReplay,
        protocol_registration_digest_v1: registration.policy_digest_v1,
        momentum_opinion_id: momentum.0.opinion_id.clone(),
        risk_opinion_id: risk.0.opinion_id.clone(),
        momentum_seal_digest_v1: momentum.1.seal_digest_v1.clone(),
        risk_seal_digest_v1: risk.1.seal_digest_v1.clone(),
        relationship,
        relationship_digest_v1: relationship_digest,
        round_count: 2,
        retrospective_only: true,
        contemporaneous_claim: false,
        chair_observed: false,
        chair_decision_created: false,
        reward_created: false,
        penalty_created: false,
        speaking_right_changed: false,
        vote_created: false,
        execution_created: false,
        transcript_digest_v1: String::new(),
    };
    let mut bytes = Vec::new();
    for text in [
        &value.deliberation_version,
        &value.protocol_registration_digest_v1,
        &value.momentum_opinion_id,
        &value.risk_opinion_id,
        &value.momentum_seal_digest_v1,
        &value.risk_seal_digest_v1,
        &value.relationship_digest_v1,
    ] {
        strv(&mut bytes, text);
    }
    usizev(&mut bytes, value.round_count);
    for flag in [
        value.retrospective_only,
        value.contemporaneous_claim,
        value.chair_observed,
        value.chair_decision_created,
        value.reward_created,
        value.penalty_created,
        value.speaking_right_changed,
        value.vote_created,
        value.execution_created,
    ] {
        boolv(&mut bytes, flag);
    }
    value.transcript_digest_v1 = stable_hash_string(&hex(&bytes));
    Ok(value)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceBoundShadowDeliberationLedgerV1 {
    pub ledger_version: String,
    pub protocol_registration_digest_v1: String,
    pub opinions: Vec<LearnedAgentOpinionEnvelopeV1>,
    pub opinion_seals: Vec<LearnedAgentOpinionSealV1>,
    pub scope_mapping_registry_digest_v1: String,
    pub legacy_v0_reference_digest: String,
    pub ledger_digest_v1: String,
}
/// The persistable, redacted form of the V1 ledger.  It retains only sealed
/// identifiers and digests; it deliberately does not serialize raw rows,
/// forecasts, probabilities, or action payloads.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceBoundShadowLedgerRecordV1 {
    pub ledger_version: String,
    pub protocol_registration_digest_v1: String,
    pub scope_mapping_registry_digest_v1: String,
    pub legacy_v0_reference_digest: String,
    pub opinions: Vec<SourceBoundLedgerOpinionRecordV1>,
    pub ledger_digest_v1: String,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceBoundLedgerOpinionRecordV1 {
    pub opinion_id: String,
    pub opinion_digest_v1: String,
    pub seal_digest_v1: String,
}
pub fn new_source_bound_shadow_ledger_v1(
    registration: &SourceBoundOpinionProtocolRegistrationV1,
    legacy_v0_reference_digest: String,
) -> Result<SourceBoundShadowDeliberationLedgerV1, String> {
    registration.validate()?;
    let mut ledger = SourceBoundShadowDeliberationLedgerV1 {
        ledger_version: "source-bound-shadow-ledger-v1".into(),
        protocol_registration_digest_v1: registration.policy_digest_v1.clone(),
        opinions: vec![],
        opinion_seals: vec![],
        scope_mapping_registry_digest_v1: String::new(),
        legacy_v0_reference_digest,
        ledger_digest_v1: String::new(),
    };
    refresh_source_bound_ledger_digest_v1(&mut ledger);
    Ok(ledger)
}
pub fn append_source_bound_opinion_v1(
    ledger: &mut SourceBoundShadowDeliberationLedgerV1,
    opinion: LearnedAgentOpinionEnvelopeV1,
    seal: LearnedAgentOpinionSealV1,
) -> Result<(), String> {
    if !opinion.sealed
        || opinion.opinion_id != seal.opinion_id
        || opinion.opinion_digest_v1 != seal.opinion_digest_v1
        || ledger
            .opinions
            .iter()
            .any(|value| value.opinion_id == opinion.opinion_id)
        || ledger
            .opinion_seals
            .iter()
            .any(|value| value.seal_digest_v1 == seal.seal_digest_v1)
    {
        return Err("source_bound_ledger_duplicate_or_invalid_opinion".into());
    }
    ledger.opinions.push(opinion);
    ledger.opinion_seals.push(seal);
    ledger
        .opinions
        .sort_by(|left, right| left.opinion_id.cmp(&right.opinion_id));
    ledger
        .opinion_seals
        .sort_by(|left, right| left.opinion_id.cmp(&right.opinion_id));
    refresh_source_bound_ledger_digest_v1(ledger);
    Ok(())
}
fn refresh_source_bound_ledger_digest_v1(ledger: &mut SourceBoundShadowDeliberationLedgerV1) {
    let mut bytes = Vec::new();
    strv(&mut bytes, &ledger.ledger_version);
    strv(&mut bytes, &ledger.protocol_registration_digest_v1);
    strv(&mut bytes, &ledger.scope_mapping_registry_digest_v1);
    strv(&mut bytes, &ledger.legacy_v0_reference_digest);
    strings(
        &mut bytes,
        &ledger
            .opinions
            .iter()
            .map(|value| value.opinion_digest_v1.clone())
            .collect::<Vec<_>>(),
    );
    strings(
        &mut bytes,
        &ledger
            .opinion_seals
            .iter()
            .map(|value| value.seal_digest_v1.clone())
            .collect::<Vec<_>>(),
    );
    ledger.ledger_digest_v1 = stable_hash_string(&hex(&bytes));
}

// Sprint 57 Phase A: these types deliberately contain only immutable input
// identities and structural policy.  They do not execute either model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointScopeSelectionPolicyV1 {
    MaximumEqualLengthChronologicalScopes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointScopeGapPolicyV1 {
    RegisteredHistoricalIsolationGap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointScopeRemainderPolicyV1 {
    ExcludeTrailingRemainder,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipantMinimumRowsV1 {
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub required_rows: usize,
    pub config_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveAnchorComparabilityPolicyV1 {
    pub minimum_shared_anchor_count: usize,
    pub minimum_shared_anchor_fraction_bits: u32,
    pub require_nonempty_train_overlap: bool,
    pub require_nonempty_validation_overlap: bool,
    pub require_nonempty_test_overlap: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointReplayAuthorityPolicyV1 {
    pub advisory_only: bool,
    pub chair_forbidden: bool,
    pub vote_forbidden: bool,
    pub reward_forbidden: bool,
    pub penalty_forbidden: bool,
    pub promotion_forbidden: bool,
    pub execution_forbidden: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointCanonicalScopeReplayRegistrationV1 {
    pub registration_version: String,
    pub replay_protocol_version: String,
    pub source_bound_opinion_protocol_digest: String,
    pub canonical_encoding_version: String,
    pub source_snapshot_id: String,
    pub source_snapshot_digest: String,
    pub provider_id: String,
    pub series_id: String,
    pub participant_agent_ids: Vec<String>,
    pub participant_objectives: Vec<LearnedAgentObjectiveV0>,
    pub participant_config_digests: Vec<String>,
    pub requested_scope_count: usize,
    pub scope_selection_policy: JointScopeSelectionPolicyV1,
    pub gap_policy: JointScopeGapPolicyV1,
    pub remainder_policy: JointScopeRemainderPolicyV1,
    pub minimum_required_rows_by_participant: Vec<ParticipantMinimumRowsV1>,
    pub joint_minimum_scope_rows: usize,
    pub retrospective_only: bool,
    pub result_dependent_scope_selection_forbidden: bool,
    pub scope_intersection_of_existing_results_forbidden: bool,
    pub model_config_changes_forbidden: bool,
    pub performance_confirmation_claims_forbidden: bool,
    pub authority_policy: JointReplayAuthorityPolicyV1,
    pub anchor_comparability_policy: EffectiveAnchorComparabilityPolicyV1,
    pub registration_digest_v1: String,
}
fn joint_registration_digest_v1(value: &JointCanonicalScopeReplayRegistrationV1) -> String {
    let mut bytes = Vec::new();
    strv(&mut bytes, "joint-canonical-scope-replay-registration-v1");
    for x in [
        &value.source_bound_opinion_protocol_digest,
        &value.source_snapshot_id,
        &value.source_snapshot_digest,
        &value.provider_id,
        &value.series_id,
    ] {
        strv(&mut bytes, x);
    }
    strings(&mut bytes, &value.participant_agent_ids);
    strings(&mut bytes, &value.participant_config_digests);
    usizev(&mut bytes, value.requested_scope_count);
    usizev(&mut bytes, value.joint_minimum_scope_rows);
    for p in &value.minimum_required_rows_by_participant {
        strv(&mut bytes, &p.agent_id);
        tag(&mut bytes, objective_tag_v1(p.objective));
        usizev(&mut bytes, p.required_rows);
        strv(&mut bytes, &p.config_digest);
    }
    for b in [
        value.retrospective_only,
        value.result_dependent_scope_selection_forbidden,
        value.scope_intersection_of_existing_results_forbidden,
        value.model_config_changes_forbidden,
        value.performance_confirmation_claims_forbidden,
        value.authority_policy.advisory_only,
        value.authority_policy.chair_forbidden,
        value.authority_policy.vote_forbidden,
        value.authority_policy.reward_forbidden,
        value.authority_policy.penalty_forbidden,
        value.authority_policy.promotion_forbidden,
        value.authority_policy.execution_forbidden,
        value
            .anchor_comparability_policy
            .require_nonempty_train_overlap,
        value
            .anchor_comparability_policy
            .require_nonempty_validation_overlap,
        value
            .anchor_comparability_policy
            .require_nonempty_test_overlap,
    ] {
        boolv(&mut bytes, b);
    }
    usizev(
        &mut bytes,
        value
            .anchor_comparability_policy
            .minimum_shared_anchor_count,
    );
    u64v(
        &mut bytes,
        value
            .anchor_comparability_policy
            .minimum_shared_anchor_fraction_bits as u64,
    );
    stable_hash_string(&hex(&bytes))
}
pub fn joint_canonical_scope_registration_v1(
    snapshot: &DataSnapshot,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<JointCanonicalScopeReplayRegistrationV1, String> {
    campaign
        .validate()
        .map_err(|_| "joint_momentum_config_invalid")?;
    let risk = CycleRiskShadowConfigV0::default();
    risk.validate().map_err(|_| "joint_risk_config_invalid")?;
    let first_momentum_window = campaign
        .train_rows
        .checked_add(campaign.purge_gap_rows)
        .and_then(|value| value.checked_add(campaign.validation_rows))
        .and_then(|value| value.checked_add(campaign.purge_gap_rows))
        .and_then(|value| value.checked_add(campaign.test_rows))
        .ok_or("joint_momentum_requirement_invalid")?;
    let momentum = campaign.minimum_history_rows.max(
        first_momentum_window.saturating_add(
            campaign
                .minimum_evaluated_windows
                .saturating_sub(1)
                .saturating_mul(campaign.step_rows),
        ),
    );
    let risk_rows =
        risk.feature.drawdown_lookback + 1 + risk.sequence_length + risk.label.horizon_rows + 48;
    let mut minimums = vec![
        ParticipantMinimumRowsV1 {
            agent_id: campaign.agent_id.clone(),
            objective: LearnedAgentObjectiveV0::DirectionalMomentum,
            required_rows: momentum,
            config_digest: campaign.digest(),
        },
        ParticipantMinimumRowsV1 {
            agent_id: CYCLE_RISK_SHADOW_AGENT_ID_V0.into(),
            objective: LearnedAgentObjectiveV0::DownsideRisk,
            required_rows: risk_rows,
            config_digest: risk.digest(),
        },
    ];
    minimums.sort_by(|a, b| a.agent_id.cmp(&b.agent_id));
    let mut value = JointCanonicalScopeReplayRegistrationV1 {
        registration_version: "joint-canonical-scope-replay-registration-v1".into(),
        replay_protocol_version: "retrospective-joint-scope-development-replay-v1".into(),
        source_bound_opinion_protocol_digest:
            SourceBoundOpinionProtocolRegistrationV1::pre_registered().policy_digest_v1,
        canonical_encoding_version: "canonical-semantic-encoding-v1".into(),
        source_snapshot_id: snapshot.snapshot_id.clone(),
        source_snapshot_digest: snapshot.content_digest.clone(),
        provider_id: snapshot.normalized_dataset.source.clone(),
        series_id: snapshot.normalized_dataset.symbol.clone(),
        participant_agent_ids: minimums.iter().map(|x| x.agent_id.clone()).collect(),
        participant_objectives: minimums.iter().map(|x| x.objective).collect(),
        participant_config_digests: minimums.iter().map(|x| x.config_digest.clone()).collect(),
        requested_scope_count: 2,
        scope_selection_policy: JointScopeSelectionPolicyV1::MaximumEqualLengthChronologicalScopes,
        gap_policy: JointScopeGapPolicyV1::RegisteredHistoricalIsolationGap,
        remainder_policy: JointScopeRemainderPolicyV1::ExcludeTrailingRemainder,
        minimum_required_rows_by_participant: minimums,
        joint_minimum_scope_rows: momentum.max(risk_rows),
        retrospective_only: true,
        result_dependent_scope_selection_forbidden: true,
        scope_intersection_of_existing_results_forbidden: true,
        model_config_changes_forbidden: true,
        performance_confirmation_claims_forbidden: true,
        authority_policy: JointReplayAuthorityPolicyV1 {
            advisory_only: true,
            chair_forbidden: true,
            vote_forbidden: true,
            reward_forbidden: true,
            penalty_forbidden: true,
            promotion_forbidden: true,
            execution_forbidden: true,
        },
        anchor_comparability_policy: EffectiveAnchorComparabilityPolicyV1 {
            minimum_shared_anchor_count: 0,
            minimum_shared_anchor_fraction_bits: 0,
            require_nonempty_train_overlap: false,
            require_nonempty_validation_overlap: false,
            require_nonempty_test_overlap: false,
        },
        registration_digest_v1: String::new(),
    };
    value.registration_digest_v1 = joint_registration_digest_v1(&value);
    Ok(value)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalRangeV1 {
    pub start_index: usize,
    pub end_index_exclusive: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeSelectionProofV1 {
    pub registration_digest_v1: String,
    pub source_snapshot_digest: String,
    pub total_source_rows: usize,
    pub requested_scope_count: usize,
    pub joint_minimum_scope_rows: usize,
    pub inter_scope_gap_rows: usize,
    pub calculated_scope_rows: usize,
    pub excluded_remainder_rows: usize,
    pub scope_ranges: Vec<CanonicalRangeV1>,
    pub non_overlapping: bool,
    pub chronology_valid: bool,
    pub all_scopes_meet_minimum: bool,
    pub result_independent: bool,
    pub all_invariants_pass: bool,
    pub proof_digest_v1: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointCanonicalHistoricalScopeV1 {
    pub scope_version: String,
    pub registration_digest_v1: String,
    pub joint_scope_id: String,
    pub chronological_rank: usize,
    pub source_snapshot_id: String,
    pub source_snapshot_digest: String,
    pub range_start_index: usize,
    pub range_end_index_exclusive: usize,
    pub canonical_raw_scope: CanonicalRawObservationScopeV1,
    pub information_cutoff_timestamp: u64,
    pub row_count: usize,
    pub selection_proof_digest_v1: String,
    pub scope_digest_v1: String,
}
pub fn issue_joint_canonical_scopes_v1(
    snapshot: &DataSnapshot,
    registration: &JointCanonicalScopeReplayRegistrationV1,
) -> Result<
    (
        JointScopeSelectionProofV1,
        Vec<JointCanonicalHistoricalScopeV1>,
    ),
    String,
> {
    if registration.registration_digest_v1 != joint_registration_digest_v1(registration)
        || registration.source_snapshot_id != snapshot.snapshot_id
        || registration.source_snapshot_digest != snapshot.content_digest
    {
        return Err("joint_registration_mismatch".into());
    }
    let gap = 0usize;
    let count = registration.requested_scope_count;
    let available = snapshot
        .normalized_dataset
        .rows
        .len()
        .saturating_sub(gap.saturating_mul(count.saturating_sub(1)));
    let length = available / count;
    let ranges = (length >= registration.joint_minimum_scope_rows)
        .then(|| {
            (0..count)
                .map(|rank| {
                    let start = rank * (length + gap);
                    CanonicalRangeV1 {
                        start_index: start,
                        end_index_exclusive: start + length,
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut proof = JointScopeSelectionProofV1 {
        registration_digest_v1: registration.registration_digest_v1.clone(),
        source_snapshot_digest: snapshot.content_digest.clone(),
        total_source_rows: snapshot.normalized_dataset.rows.len(),
        requested_scope_count: count,
        joint_minimum_scope_rows: registration.joint_minimum_scope_rows,
        inter_scope_gap_rows: gap,
        calculated_scope_rows: length,
        excluded_remainder_rows: available.saturating_sub(length * count),
        scope_ranges: ranges.clone(),
        non_overlapping: true,
        chronology_valid: true,
        all_scopes_meet_minimum: ranges.len() == count,
        result_independent: true,
        all_invariants_pass: ranges.len() == count,
        proof_digest_v1: String::new(),
    };
    proof.proof_digest_v1 = stable_hash_string(&format!(
        "{}:{}:{}:{}",
        proof.registration_digest_v1,
        proof.total_source_rows,
        proof.calculated_scope_rows,
        proof.excluded_remainder_rows
    ));
    let scopes = ranges
        .into_iter()
        .enumerate()
        .map(|(rank, range)| {
            let raw = canonical_raw_scope_v1(
                snapshot,
                range.start_index,
                range.end_index_exclusive,
                &registration.registration_digest_v1,
            )?;
            let digest = stable_hash_string(&format!(
                "{}:{}:{}",
                proof.proof_digest_v1, rank, raw.scope_digest_v1
            ));
            Ok(JointCanonicalHistoricalScopeV1 {
                scope_version: "joint-canonical-historical-scope-v1".into(),
                registration_digest_v1: registration.registration_digest_v1.clone(),
                joint_scope_id: format!("joint-scope-{rank}"),
                chronological_rank: rank,
                source_snapshot_id: snapshot.snapshot_id.clone(),
                source_snapshot_digest: snapshot.content_digest.clone(),
                range_start_index: range.start_index,
                range_end_index_exclusive: range.end_index_exclusive,
                information_cutoff_timestamp: raw.information_cutoff_timestamp,
                row_count: raw.row_count,
                canonical_raw_scope: raw,
                selection_proof_digest_v1: proof.proof_digest_v1.clone(),
                scope_digest_v1: digest,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok((proof, scopes))
}

fn replay_local_joint_scope_snapshot_v1(
    snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
) -> DataSnapshot {
    let mut local = snapshot.clone();
    local.normalized_dataset.rows = snapshot.normalized_dataset.rows
        [scope.range_start_index..scope.range_end_index_exclusive]
        .to_vec();
    local.row_count = local.normalized_dataset.rows.len();
    local.actual_start_timestamp_ms = local
        .normalized_dataset
        .rows
        .first()
        .map(|row| row.timestamp_ms);
    local.actual_end_timestamp_ms = local
        .normalized_dataset
        .rows
        .last()
        .map(|row| row.timestamp_ms);
    local.content_digest =
        crate::data::historical_replay_dataset_digest_v0(&local.normalized_dataset);
    local
}

fn replay_local_joint_momentum_pack_v1(
    snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
) -> Result<
    (
        BtcHistoricalRegimeV0,
        super::MomentumHistoricalEvidencePackV0,
    ),
    String,
> {
    let local = replay_local_joint_scope_snapshot_v1(snapshot, scope);
    let (_, pack) = super::freeze_momentum_historical_evidence_pack_v0(
        &[local.clone()],
        &HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| "joint_momentum_pack_invalid")?;
    Ok((
        BtcHistoricalRegimeV0 {
            regime_id: scope.joint_scope_id.clone(),
            start_row_index: 0,
            end_row_index_exclusive: local.row_count,
            start_timestamp_ms: local
                .actual_start_timestamp_ms
                .ok_or("joint_scope_start_missing")?,
            end_timestamp_ms: local
                .actual_end_timestamp_ms
                .ok_or("joint_scope_end_missing")?,
            row_count: local.row_count,
            source_snapshot_id: local.snapshot_id,
            usage_class: EvidenceUsageClassV0::DevelopmentEligible,
            segmentation_config_digest: scope.registration_digest_v1.clone(),
        },
        pack,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointScopeParticipantReplayStatusV1 {
    Completed,
    NoUsableValidationSignal,
    InsufficientHistory,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeReplayResultV1 {
    pub joint_scope_id: String,
    pub joint_scope_digest_v1: String,
    pub momentum_result_digest_v1: String,
    pub risk_result_digest_v1: String,
    pub momentum_anchor_scope_digest_v1: String,
    pub risk_anchor_scope_digest_v1: String,
    pub momentum_status: JointScopeParticipantReplayStatusV1,
    pub risk_status: JointScopeParticipantReplayStatusV1,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeOpinionPairV1 {
    pub joint_scope_id: String,
    pub momentum: (LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1),
    pub risk: (LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1),
}
pub fn replay_joint_scope_results_v1(
    snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<JointScopeReplayResultV1, String> {
    let local = replay_local_joint_scope_snapshot_v1(snapshot, scope);
    let pack = replay_local_joint_momentum_pack_v1(snapshot, scope)?;
    let encoder = frozen_mamba3_encoder_from_seed_v0(
        &campaign.feature_config,
        campaign.campaign_seed,
        campaign.backend_preference,
        campaign.fallback_policy,
    )
    .map_err(|_| "joint_momentum_encoder_invalid")?;
    let momentum = run_btc_historical_regime_campaigns_v0(&[pack], campaign, &encoder)
        .ok()
        .and_then(|mut values| values.pop());
    let risk = CycleRiskShadowConfigV0::default();
    let risk_result = run_cycle_risk_shadow_regime_v0(&local, &scope.joint_scope_id, &risk).ok();
    let momentum_anchors = momentum_anchor_scope_v1(&local, 0, local.row_count, campaign).ok();
    let risk_pack_digest = risk_result
        .as_ref()
        .map(|result| result.frozen_pack_digest.clone())
        .unwrap_or_default();
    let candidate = CycleRiskHistoricalRangeCandidateV0 {
        candidate_range_id: scope.joint_scope_id.clone(),
        start_row_index: 0,
        end_row_index_exclusive: local.row_count,
        row_count: local.row_count,
        canonical_scope_digest_v1: scope.canonical_raw_scope.scope_digest_v1.clone(),
        expected_frozen_pack_digest: risk_pack_digest,
        range_digest: scope.scope_digest_v1.clone(),
    };
    let risk_anchors = risk_anchor_scope_v1(&local, &candidate, &risk).ok();
    Ok(JointScopeReplayResultV1 {
        joint_scope_id: scope.joint_scope_id.clone(),
        joint_scope_digest_v1: scope.scope_digest_v1.clone(),
        momentum_result_digest_v1: momentum
            .as_ref()
            .map(|result| result.report_digest.clone())
            .unwrap_or_default(),
        risk_result_digest_v1: risk_result
            .as_ref()
            .map(|result| stable_hash_string(&format!("{:?}", result)))
            .unwrap_or_default(),
        momentum_anchor_scope_digest_v1: momentum_anchors
            .as_ref()
            .map(|value| value.scope_digest_v1.clone())
            .unwrap_or_default(),
        risk_anchor_scope_digest_v1: risk_anchors
            .as_ref()
            .map(|value| value.scope_digest_v1.clone())
            .unwrap_or_default(),
        momentum_status: if momentum.is_some() && momentum_anchors.is_some() {
            JointScopeParticipantReplayStatusV1::Completed
        } else {
            JointScopeParticipantReplayStatusV1::NoUsableValidationSignal
        },
        risk_status: if risk_result.is_some() && risk_anchors.is_some() {
            JointScopeParticipantReplayStatusV1::Completed
        } else {
            JointScopeParticipantReplayStatusV1::InsufficientHistory
        },
    })
}

// Sprint 58 deliberately keeps the Sprint 57 V1 replay above intact.  V2
// records a separate forensic and replay protocol so legacy output is not
// reinterpreted or rewritten.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointParticipantExecutionHealthV2 {
    Completed,
    ParentSnapshotInvalid,
    JointScopeInvalid,
    DerivedSnapshotConstructionFailure,
    DerivedSnapshotSemanticFailure,
    EvidencePolicyFailure,
    InventoryRejected,
    NoAcceptedSeries,
    PackConstructionFailure,
    PackVerificationFailure,
    EncoderConstructionFailure,
    CampaignConfigurationFailure,
    CampaignRuntimeFailure,
    CampaignOutputMissing,
    AnchorMaterializationFailure,
    ResultClosureFailure,
    NondeterministicReplay,
    TechnicalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointParticipantModelEvidenceOutcomeV2 {
    UsableValidationSignal,
    NoUsableValidationSignal,
    ValidationSignalOutOfSupport,
    InsufficientEvidence,
    ProbabilityCollapse,
    RepresentationShiftRisk,
    BaselineStronger,
    NotEvaluatedTechnicalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointParticipantOperationalShadowResultV2 {
    ShadowPredictionResearchOnly,
    ShadowAbstainNoSignal,
    ShadowAbstainOutOfSupport,
    ShadowAbstainInsufficientEvidence,
    ShadowAbstainTechnicalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointAnchorAuditStatusV2 {
    Complete,
    CompleteWithoutSelectedCheckpoint,
    NoValidExamples,
    FeatureConstructionFailure,
    SequenceConstructionFailure,
    WindowConstructionFailure,
    PartitionIdentityFailure,
    TechnicalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointParticipantExecutionStageV2 {
    ParentSnapshotVerification,
    JointScopeVerification,
    DerivedSnapshotConstruction,
    DerivedSnapshotIdentity,
    DerivedSnapshotSemanticVerification,
    EvidencePolicyConstruction,
    SnapshotInventoryClassification,
    EvidencePackConstruction,
    EvidencePackVerification,
    EncoderConstruction,
    FeatureExtraction,
    SequenceConstruction,
    WindowConstruction,
    CampaignExecution,
    ValidationSignalGate,
    CheckpointSelection,
    TemporalDiagnostics,
    ResultClosure,
    AnchorMaterialization,
    SourceBoundOpinionConstruction,
    OpinionSeal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointExecutionStageStatusV2 {
    Completed,
    CompletedNoSignal,
    CompletedAbstained,
    NotApplicable,
    NotExecutedAfterFailure,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointExecutionStageResultV2 {
    pub stage: JointParticipantExecutionStageV2,
    pub status: JointExecutionStageStatusV2,
    pub sanitized_error_code: Option<String>,
    pub reason_codes: Vec<String>,
    pub artifact_digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointParticipantExecutionTraceV2 {
    pub trace_version: String,
    pub joint_scope_id: String,
    pub participant_agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub stages: Vec<JointExecutionStageResultV2>,
    pub first_failed_stage: Option<JointParticipantExecutionStageV2>,
    pub execution_health: JointParticipantExecutionHealthV2,
    pub model_evidence_outcome: JointParticipantModelEvidenceOutcomeV2,
    pub operational_shadow_result: JointParticipantOperationalShadowResultV2,
    pub trace_digest_v2: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeSnapshotDerivationProofV2 {
    pub parent_verified: bool,
    pub exact_registered_row_subset: bool,
    pub provider_preserved: bool,
    pub series_preserved: bool,
    pub chronology_valid: bool,
    pub derived_row_count_consistent: bool,
    pub quality_row_count_consistent: bool,
    pub symbol_metadata_consistent: bool,
    pub timestamp_metadata_consistent: bool,
    pub content_digest_consistent: bool,
    pub snapshot_identity_consistent: bool,
    pub read_only_preserved: bool,
    pub sanitized_preserved: bool,
    pub credential_free_preserved: bool,
    pub immutable_reason_preserved: bool,
    pub all_invariants_pass: bool,
    pub proof_digest_v2: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeSnapshotLineageV2 {
    pub parent_snapshot_id: String,
    pub parent_snapshot_digest: String,
    pub derived_snapshot_id: String,
    pub derived_snapshot_digest: String,
    pub joint_scope_id: String,
    pub joint_scope_digest: String,
    pub lineage_digest_v2: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JointScopeDerivedSnapshotV2 {
    pub derivation_version: String,
    pub parent_snapshot_id: String,
    pub parent_snapshot_digest: String,
    pub joint_scope_id: String,
    pub joint_scope_digest: String,
    pub derived_snapshot: DataSnapshot,
    pub derivation_proof: JointScopeSnapshotDerivationProofV2,
    pub lineage: JointScopeSnapshotLineageV2,
    pub derivation_digest_v2: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeDerivedEvidencePolicyV2 {
    pub parent_snapshot_id: String,
    pub derived_snapshot_id: String,
    pub derivation_proof_digest: String,
    pub exact_child_authorized: bool,
    pub wildcard_authorization: bool,
    pub policy_digest_v2: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointCanonicalScopeReplayRegistrationV2 {
    pub registration_version: String,
    pub parent_registration_digest_v1: String,
    pub source_bound_opinion_protocol_digest: String,
    pub canonical_encoding_version: String,
    pub joint_scope_ids: Vec<String>,
    pub joint_scope_digests: Vec<String>,
    pub derived_snapshot_policy_digest: String,
    pub derived_evidence_policy_digest: String,
    pub execution_trace_policy_digest: String,
    pub participant_status_policy_digest: String,
    pub completed_no_signal_opinion_policy_digest: String,
    pub scope_ranges_unchanged: bool,
    pub scope_selection_unchanged: bool,
    pub participant_configs_unchanged: bool,
    pub result_dependent_changes_forbidden: bool,
    pub authority_policy_digest: String,
    pub registration_digest_v2: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointMomentumRootCauseV2 {
    DerivedSnapshotMetadataMismatch,
    DerivedSnapshotIdentityMismatch,
    DerivedEvidenceApprovalMissing,
    InventoryRejectedExpectedChild,
    EmptyAcceptedSeries,
    PackInvariantFailure,
    CampaignConfigurationFailure,
    CampaignRuntimeFailure,
    GenuineCompletedNoUsableValidationSignal,
    AnchorAuditFailureAfterCompletedCampaign,
    MultipleCauses,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sprint57MomentumOutcomeInterpretationV2 {
    pub sprint57_registration_digest: String,
    pub joint_scope_id: String,
    pub legacy_reported_status: String,
    pub forensic_root_cause: JointMomentumRootCauseV2,
    pub corrected_execution_health: JointParticipantExecutionHealthV2,
    pub corrected_model_outcome: JointParticipantModelEvidenceOutcomeV2,
    pub sprint57_artifact_mutated: bool,
    pub interpretation_digest_v2: String,
}

fn joint_v2_digest(parts: &[String]) -> String {
    let mut bytes = Vec::new();
    strv(&mut bytes, "joint-canonical-scope-replay-v2");
    strings(&mut bytes, parts);
    stable_hash_string(&hex(&bytes))
}

fn joint_v2_policy_digest(label: &str, parts: &[&str]) -> String {
    let mut values = vec![label.to_string()];
    values.extend(parts.iter().map(|part| (*part).to_string()));
    joint_v2_digest(&values)
}

fn joint_v2_registration_digest(value: &JointCanonicalScopeReplayRegistrationV2) -> String {
    let mut values = vec![
        value.registration_version.clone(),
        value.parent_registration_digest_v1.clone(),
        value.source_bound_opinion_protocol_digest.clone(),
        value.canonical_encoding_version.clone(),
        value.derived_snapshot_policy_digest.clone(),
        value.derived_evidence_policy_digest.clone(),
        value.execution_trace_policy_digest.clone(),
        value.participant_status_policy_digest.clone(),
        value.completed_no_signal_opinion_policy_digest.clone(),
        value.authority_policy_digest.clone(),
    ];
    values.extend(value.joint_scope_ids.clone());
    values.extend(value.joint_scope_digests.clone());
    values.extend([
        value.scope_ranges_unchanged.to_string(),
        value.scope_selection_unchanged.to_string(),
        value.participant_configs_unchanged.to_string(),
        value.result_dependent_changes_forbidden.to_string(),
    ]);
    joint_v2_digest(&values)
}

pub fn joint_canonical_scope_registration_v2(
    snapshot: &DataSnapshot,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<JointCanonicalScopeReplayRegistrationV2, String> {
    let parent = joint_canonical_scope_registration_v1(snapshot, campaign)?;
    let (proof, scopes) = issue_joint_canonical_scopes_v1(snapshot, &parent)?;
    if !proof.all_invariants_pass || scopes.len() != parent.requested_scope_count {
        return Err("joint_v2_scope_registration_unavailable".into());
    }
    let scope_ids = scopes
        .iter()
        .map(|scope| scope.joint_scope_id.clone())
        .collect::<Vec<_>>();
    let scope_digests = scopes
        .iter()
        .map(|scope| scope.scope_digest_v1.clone())
        .collect::<Vec<_>>();
    let mut value = JointCanonicalScopeReplayRegistrationV2 {
        registration_version: "joint-canonical-scope-replay-registration-v2".into(),
        parent_registration_digest_v1: parent.registration_digest_v1,
        source_bound_opinion_protocol_digest:
            SourceBoundOpinionProtocolRegistrationV1::pre_registered().policy_digest_v1,
        canonical_encoding_version: "canonical-semantic-encoding-v1".into(),
        joint_scope_ids: scope_ids,
        joint_scope_digests: scope_digests,
        derived_snapshot_policy_digest: joint_v2_policy_digest(
            "derived-snapshot-policy-v2",
            &[
                "child-semantic-identity",
                "coupled-metadata",
                "parent-lineage",
            ],
        ),
        derived_evidence_policy_digest: joint_v2_policy_digest(
            "derived-evidence-policy-v2",
            &[
                "exact-child-only",
                "wildcard-forbidden",
                "global-policy-unchanged",
            ],
        ),
        execution_trace_policy_digest: joint_v2_policy_digest(
            "execution-trace-policy-v2",
            &[
                "typed-stage-results",
                "no-error-swallowing",
                "sanitized-errors",
            ],
        ),
        participant_status_policy_digest: joint_v2_policy_digest(
            "participant-status-policy-v2",
            &[
                "execution-model-anchor-separated",
                "technical-failure-not-no-signal",
            ],
        ),
        completed_no_signal_opinion_policy_digest: joint_v2_policy_digest(
            "completed-no-signal-opinion-policy-v2",
            &["completed-only", "source-bound-abstain", "authority-false"],
        ),
        scope_ranges_unchanged: true,
        scope_selection_unchanged: true,
        participant_configs_unchanged: true,
        result_dependent_changes_forbidden: true,
        authority_policy_digest: joint_v2_policy_digest(
            "joint-authority-policy-v2",
            &[
                "advisory-only",
                "chair-vote-reward-penalty-promotion-execution-forbidden",
            ],
        ),
        registration_digest_v2: String::new(),
    };
    value.registration_digest_v2 = joint_v2_registration_digest(&value);
    Ok(value)
}

pub fn validate_joint_canonical_scope_registration_v2(
    snapshot: &DataSnapshot,
    campaign: &MomentumLearningCampaignConfigV0,
    registration: &JointCanonicalScopeReplayRegistrationV2,
) -> Result<Vec<JointCanonicalHistoricalScopeV1>, String> {
    if registration.registration_version != "joint-canonical-scope-replay-registration-v2"
        || registration.registration_digest_v2 != joint_v2_registration_digest(registration)
        || !registration.scope_ranges_unchanged
        || !registration.scope_selection_unchanged
        || !registration.participant_configs_unchanged
        || !registration.result_dependent_changes_forbidden
    {
        return Err("joint_v2_registration_invalid".into());
    }
    let parent = joint_canonical_scope_registration_v1(snapshot, campaign)?;
    let (proof, scopes) = issue_joint_canonical_scopes_v1(snapshot, &parent)?;
    if !proof.all_invariants_pass
        || parent.registration_digest_v1 != registration.parent_registration_digest_v1
        || scopes
            .iter()
            .map(|scope| scope.joint_scope_id.clone())
            .collect::<Vec<_>>()
            != registration.joint_scope_ids
        || scopes
            .iter()
            .map(|scope| scope.scope_digest_v1.clone())
            .collect::<Vec<_>>()
            != registration.joint_scope_digests
    {
        return Err("joint_v2_scope_reuse_mismatch".into());
    }
    Ok(scopes)
}

fn derived_snapshot_proof_digest_v2(value: &JointScopeSnapshotDerivationProofV2) -> String {
    joint_v2_digest(&[
        value.parent_verified.to_string(),
        value.exact_registered_row_subset.to_string(),
        value.provider_preserved.to_string(),
        value.series_preserved.to_string(),
        value.chronology_valid.to_string(),
        value.derived_row_count_consistent.to_string(),
        value.quality_row_count_consistent.to_string(),
        value.symbol_metadata_consistent.to_string(),
        value.timestamp_metadata_consistent.to_string(),
        value.content_digest_consistent.to_string(),
        value.snapshot_identity_consistent.to_string(),
        value.read_only_preserved.to_string(),
        value.sanitized_preserved.to_string(),
        value.credential_free_preserved.to_string(),
        value.immutable_reason_preserved.to_string(),
    ])
}

fn v2_scope_matches_snapshot(
    snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
) -> Result<(), String> {
    if scope.source_snapshot_id != snapshot.snapshot_id
        || scope.source_snapshot_digest != snapshot.content_digest
        || scope.range_start_index >= scope.range_end_index_exclusive
        || scope.range_end_index_exclusive > snapshot.normalized_dataset.rows.len()
        || scope.row_count != scope.range_end_index_exclusive - scope.range_start_index
    {
        return Err("joint_scope_snapshot_mismatch".into());
    }
    let raw = canonical_raw_scope_v1(
        snapshot,
        scope.range_start_index,
        scope.range_end_index_exclusive,
        &scope.registration_digest_v1,
    )?;
    if raw != scope.canonical_raw_scope
        || raw.information_cutoff_timestamp != scope.information_cutoff_timestamp
    {
        return Err("joint_scope_raw_identity_mismatch".into());
    }
    Ok(())
}

pub fn derive_joint_scope_snapshot_v2(
    snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
) -> Result<JointScopeDerivedSnapshotV2, String> {
    v2_scope_matches_snapshot(snapshot, scope)?;
    let parent_digest = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset);
    let parent_verified = parent_digest == snapshot.content_digest
        && snapshot.snapshot_id == snapshot_id_from_semantic_digest_v1(&parent_digest)
        && snapshot.row_count == snapshot.normalized_dataset.rows.len()
        && snapshot.quality_summary.row_count == snapshot.row_count
        && snapshot.symbols == vec![snapshot.normalized_dataset.symbol.clone()]
        && snapshot.actual_start_timestamp_ms
            == snapshot
                .normalized_dataset
                .rows
                .first()
                .map(|row| row.timestamp_ms)
        && snapshot.actual_end_timestamp_ms
            == snapshot
                .normalized_dataset
                .rows
                .last()
                .map(|row| row.timestamp_ms);
    if !parent_verified {
        return Err("joint_parent_snapshot_semantic_invalid".into());
    }

    let rows = snapshot.normalized_dataset.rows
        [scope.range_start_index..scope.range_end_index_exclusive]
        .to_vec();
    let first = rows.first().map(|row| row.timestamp_ms);
    let last = rows.last().map(|row| row.timestamp_ms);
    let mut derived = snapshot.clone();
    derived.normalized_dataset.rows = rows;
    derived.row_count = derived.normalized_dataset.rows.len();
    derived.quality_summary.row_count = derived.row_count;
    derived.requested_lookback.bars = derived.row_count;
    derived.requested_lookback.start_timestamp_ms = first;
    derived.requested_lookback.end_timestamp_ms = last;
    derived.actual_start_timestamp_ms = first;
    derived.actual_end_timestamp_ms = last;
    derived.content_digest = historical_replay_dataset_digest_v0(&derived.normalized_dataset);
    derived.snapshot_id = snapshot_id_from_semantic_digest_v1(&derived.content_digest);
    derived.request_key = format!(
        "joint-derived-v2-{}",
        stable_hash_string(&format!(
            "{}:{}:{}",
            snapshot.snapshot_id, scope.joint_scope_id, scope.scope_digest_v1
        ))
    );
    derived.provenance.acquisition_request_id = derived.request_key.clone();

    let chronology_valid = derived
        .normalized_dataset
        .rows
        .windows(2)
        .all(|pair| pair[0].timestamp_ms < pair[1].timestamp_ms);
    let immutable_reason_preserved = snapshot
        .reason_codes
        .iter()
        .any(|code| matches!(code, crate::core::ReasonCode::DataSnapshotImmutable))
        && derived.reason_codes == snapshot.reason_codes;
    let mut proof = JointScopeSnapshotDerivationProofV2 {
        parent_verified,
        exact_registered_row_subset: derived.normalized_dataset.rows
            == snapshot.normalized_dataset.rows
                [scope.range_start_index..scope.range_end_index_exclusive],
        provider_preserved: derived.provider_id == snapshot.provider_id
            && derived.provenance.provider_id == snapshot.provenance.provider_id,
        series_preserved: derived.normalized_dataset.symbol == snapshot.normalized_dataset.symbol,
        chronology_valid,
        derived_row_count_consistent: derived.row_count == derived.normalized_dataset.rows.len(),
        quality_row_count_consistent: derived.quality_summary.row_count == derived.row_count
            && derived.quality_summary.accepted == snapshot.quality_summary.accepted,
        symbol_metadata_consistent: derived.symbols
            == vec![derived.normalized_dataset.symbol.clone()],
        timestamp_metadata_consistent: derived.actual_start_timestamp_ms == first
            && derived.actual_end_timestamp_ms == last,
        content_digest_consistent: derived.content_digest
            == historical_replay_dataset_digest_v0(&derived.normalized_dataset),
        snapshot_identity_consistent: derived.snapshot_id
            == snapshot_id_from_semantic_digest_v1(&derived.content_digest),
        read_only_preserved: derived.read_only == snapshot.read_only && derived.read_only,
        sanitized_preserved: derived.sanitized == snapshot.sanitized
            && derived.provenance.sanitized == snapshot.provenance.sanitized
            && derived.sanitized,
        credential_free_preserved: derived.provenance.credential_free
            == snapshot.provenance.credential_free
            && derived.provenance.credential_free,
        immutable_reason_preserved,
        all_invariants_pass: false,
        proof_digest_v2: String::new(),
    };
    proof.all_invariants_pass = proof.parent_verified
        && proof.exact_registered_row_subset
        && proof.provider_preserved
        && proof.series_preserved
        && proof.chronology_valid
        && proof.derived_row_count_consistent
        && proof.quality_row_count_consistent
        && proof.symbol_metadata_consistent
        && proof.timestamp_metadata_consistent
        && proof.content_digest_consistent
        && proof.snapshot_identity_consistent
        && proof.read_only_preserved
        && proof.sanitized_preserved
        && proof.credential_free_preserved
        && proof.immutable_reason_preserved;
    proof.proof_digest_v2 = derived_snapshot_proof_digest_v2(&proof);
    if !proof.all_invariants_pass {
        return Err("joint_derived_snapshot_invariant_failed".into());
    }
    let lineage = JointScopeSnapshotLineageV2 {
        parent_snapshot_id: snapshot.snapshot_id.clone(),
        parent_snapshot_digest: snapshot.content_digest.clone(),
        derived_snapshot_id: derived.snapshot_id.clone(),
        derived_snapshot_digest: derived.content_digest.clone(),
        joint_scope_id: scope.joint_scope_id.clone(),
        joint_scope_digest: scope.scope_digest_v1.clone(),
        lineage_digest_v2: joint_v2_digest(&[
            snapshot.snapshot_id.clone(),
            snapshot.content_digest.clone(),
            derived.snapshot_id.clone(),
            derived.content_digest.clone(),
            scope.joint_scope_id.clone(),
            scope.scope_digest_v1.clone(),
        ]),
    };
    let derivation_digest_v2 = joint_v2_digest(&[
        "joint-scope-derived-snapshot-v2".into(),
        proof.proof_digest_v2.clone(),
        lineage.lineage_digest_v2.clone(),
    ]);
    Ok(JointScopeDerivedSnapshotV2 {
        derivation_version: "joint-scope-derived-snapshot-v2".into(),
        parent_snapshot_id: snapshot.snapshot_id.clone(),
        parent_snapshot_digest: snapshot.content_digest.clone(),
        joint_scope_id: scope.joint_scope_id.clone(),
        joint_scope_digest: scope.scope_digest_v1.clone(),
        derived_snapshot: derived,
        derivation_proof: proof,
        lineage,
        derivation_digest_v2,
    })
}

pub fn joint_scope_derived_evidence_policy_v2(
    value: &JointScopeDerivedSnapshotV2,
) -> Result<JointScopeDerivedEvidencePolicyV2, String> {
    if !value.derivation_proof.all_invariants_pass
        || value.parent_snapshot_id.is_empty()
        || value.derived_snapshot.snapshot_id.is_empty()
    {
        return Err("joint_derived_evidence_policy_invalid".into());
    }
    let mut policy = JointScopeDerivedEvidencePolicyV2 {
        parent_snapshot_id: value.parent_snapshot_id.clone(),
        derived_snapshot_id: value.derived_snapshot.snapshot_id.clone(),
        derivation_proof_digest: value.derivation_proof.proof_digest_v2.clone(),
        exact_child_authorized: true,
        wildcard_authorization: false,
        policy_digest_v2: String::new(),
    };
    policy.policy_digest_v2 = joint_v2_digest(&[
        policy.parent_snapshot_id.clone(),
        policy.derived_snapshot_id.clone(),
        policy.derivation_proof_digest.clone(),
        policy.exact_child_authorized.to_string(),
        policy.wildcard_authorization.to_string(),
    ]);
    Ok(policy)
}

fn execution_trace_digest_v2(value: &JointParticipantExecutionTraceV2) -> String {
    let mut parts = vec![
        value.trace_version.clone(),
        value.joint_scope_id.clone(),
        value.participant_agent_id.clone(),
        format!("{:?}", value.objective),
        format!("{:?}", value.execution_health),
        format!("{:?}", value.model_evidence_outcome),
        format!("{:?}", value.operational_shadow_result),
    ];
    parts.extend(value.stages.iter().map(|stage| {
        format!(
            "{:?}:{:?}:{}:{}:{}",
            stage.stage,
            stage.status,
            stage.sanitized_error_code.clone().unwrap_or_default(),
            stage.reason_codes.join(","),
            stage.artifact_digest.clone().unwrap_or_default(),
        )
    }));
    joint_v2_digest(&parts)
}

fn new_execution_trace_v2(
    scope: &JointCanonicalHistoricalScopeV1,
    participant_agent_id: String,
    objective: LearnedAgentObjectiveV0,
) -> JointParticipantExecutionTraceV2 {
    JointParticipantExecutionTraceV2 {
        trace_version: "joint-participant-execution-trace-v2".into(),
        joint_scope_id: scope.joint_scope_id.clone(),
        participant_agent_id,
        objective,
        stages: vec![],
        first_failed_stage: None,
        execution_health: JointParticipantExecutionHealthV2::TechnicalFailure,
        model_evidence_outcome:
            JointParticipantModelEvidenceOutcomeV2::NotEvaluatedTechnicalFailure,
        operational_shadow_result:
            JointParticipantOperationalShadowResultV2::ShadowAbstainTechnicalFailure,
        trace_digest_v2: String::new(),
    }
}

fn record_execution_stage_v2(
    trace: &mut JointParticipantExecutionTraceV2,
    stage: JointParticipantExecutionStageV2,
    status: JointExecutionStageStatusV2,
    sanitized_error_code: Option<&str>,
    reason_codes: Vec<String>,
    artifact_digest: Option<String>,
) {
    if status == JointExecutionStageStatusV2::Failed && trace.first_failed_stage.is_none() {
        trace.first_failed_stage = Some(stage);
    }
    trace.stages.push(JointExecutionStageResultV2 {
        stage,
        status,
        sanitized_error_code: sanitized_error_code.map(str::to_string),
        reason_codes,
        artifact_digest,
    });
}

fn finish_execution_trace_v2(trace: &mut JointParticipantExecutionTraceV2) {
    trace.trace_digest_v2 = execution_trace_digest_v2(trace);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointMomentumForensicReportV2 {
    pub report_version: String,
    pub joint_scope_id: String,
    pub derived_snapshot_id: String,
    pub derived_snapshot_digest: String,
    pub quality_summary_consistent: bool,
    pub accepted_series_count: usize,
    pub rejected_snapshot_count: usize,
    pub pack_series_count: usize,
    pub campaign_invocation_status: String,
    pub anchor_invocation_status: String,
    pub execution_trace: JointParticipantExecutionTraceV2,
    pub root_cause: JointMomentumRootCauseV2,
    pub forensic_digest_v2: String,
}

fn legacy_scope_snapshot_integrity_v2(
    snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
) -> Result<(DataSnapshot, bool, bool), String> {
    v2_scope_matches_snapshot(snapshot, scope)?;
    let legacy = replay_local_joint_scope_snapshot_v1(snapshot, scope);
    let quality_consistent = legacy.quality_summary.row_count == legacy.row_count
        && legacy.quality_summary.accepted == snapshot.quality_summary.accepted;
    let semantic_identity_consistent =
        legacy.snapshot_id == snapshot_id_from_semantic_digest_v1(&legacy.content_digest);
    Ok((legacy, quality_consistent, semantic_identity_consistent))
}

pub fn forensic_joint_momentum_scope_v2(
    snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<JointMomentumForensicReportV2, String> {
    let mut trace = new_execution_trace_v2(
        scope,
        campaign.agent_id.clone(),
        LearnedAgentObjectiveV0::DirectionalMomentum,
    );
    let parent_valid = historical_replay_dataset_digest_v0(&snapshot.normalized_dataset)
        == snapshot.content_digest
        && snapshot.row_count == snapshot.normalized_dataset.rows.len()
        && snapshot.quality_summary.row_count == snapshot.row_count;
    record_execution_stage_v2(
        &mut trace,
        JointParticipantExecutionStageV2::ParentSnapshotVerification,
        if parent_valid {
            JointExecutionStageStatusV2::Completed
        } else {
            JointExecutionStageStatusV2::Failed
        },
        (!parent_valid).then_some("parent_snapshot_semantic_invalid"),
        vec![format!("parent_verified={parent_valid}")],
        Some(snapshot.content_digest.clone()),
    );
    if !parent_valid {
        trace.execution_health = JointParticipantExecutionHealthV2::ParentSnapshotInvalid;
        finish_execution_trace_v2(&mut trace);
        return Ok(forensic_report_v2(
            scope,
            snapshot.snapshot_id.clone(),
            snapshot.content_digest.clone(),
            false,
            0,
            0,
            0,
            "not_executed_after_parent_failure".into(),
            "not_executed_after_parent_failure".into(),
            trace,
            JointMomentumRootCauseV2::Unknown,
        ));
    }
    let scope_valid = v2_scope_matches_snapshot(snapshot, scope).is_ok();
    record_execution_stage_v2(
        &mut trace,
        JointParticipantExecutionStageV2::JointScopeVerification,
        if scope_valid {
            JointExecutionStageStatusV2::Completed
        } else {
            JointExecutionStageStatusV2::Failed
        },
        (!scope_valid).then_some("joint_scope_raw_identity_mismatch"),
        vec![format!("scope_verified={scope_valid}")],
        Some(scope.scope_digest_v1.clone()),
    );
    if !scope_valid {
        trace.execution_health = JointParticipantExecutionHealthV2::JointScopeInvalid;
        finish_execution_trace_v2(&mut trace);
        return Ok(forensic_report_v2(
            scope,
            snapshot.snapshot_id.clone(),
            snapshot.content_digest.clone(),
            false,
            0,
            0,
            0,
            "not_executed_after_scope_failure".into(),
            "not_executed_after_scope_failure".into(),
            trace,
            JointMomentumRootCauseV2::Unknown,
        ));
    }
    let (legacy, quality_summary_consistent, identity_consistent) =
        legacy_scope_snapshot_integrity_v2(snapshot, scope)?;
    record_execution_stage_v2(
        &mut trace,
        JointParticipantExecutionStageV2::DerivedSnapshotConstruction,
        JointExecutionStageStatusV2::Completed,
        None,
        vec!["legacy_subset_constructed=true".into()],
        Some(legacy.content_digest.clone()),
    );
    record_execution_stage_v2(
        &mut trace,
        JointParticipantExecutionStageV2::DerivedSnapshotIdentity,
        if identity_consistent {
            JointExecutionStageStatusV2::Completed
        } else {
            JointExecutionStageStatusV2::Failed
        },
        (!identity_consistent).then_some("legacy_child_impersonates_parent_snapshot"),
        vec![format!(
            "semantic_identity_consistent={identity_consistent}"
        )],
        Some(legacy.snapshot_id.clone()),
    );
    record_execution_stage_v2(
        &mut trace,
        JointParticipantExecutionStageV2::DerivedSnapshotSemanticVerification,
        if quality_summary_consistent {
            JointExecutionStageStatusV2::Completed
        } else {
            JointExecutionStageStatusV2::Failed
        },
        (!quality_summary_consistent).then_some("legacy_quality_row_count_mismatch"),
        vec![format!(
            "quality_summary_consistent={quality_summary_consistent}"
        )],
        Some(legacy.content_digest.clone()),
    );

    let policy = HistoricalEvidencePolicyV0::default();
    let inventory =
        super::inventory_historical_snapshots_v0(std::slice::from_ref(&legacy), &policy)
            .map_err(|_| "forensic_inventory_construction_failed".to_string())?;
    let accepted_series_count = inventory.accepted_series.len();
    let rejected_snapshot_count = inventory.rejected_snapshots.len();
    record_execution_stage_v2(
        &mut trace,
        JointParticipantExecutionStageV2::SnapshotInventoryClassification,
        if accepted_series_count == 1 {
            JointExecutionStageStatusV2::Completed
        } else {
            JointExecutionStageStatusV2::Failed
        },
        (accepted_series_count != 1).then_some("legacy_inventory_expected_child_not_accepted"),
        vec![
            format!("accepted_series_count={accepted_series_count}"),
            format!("rejected_snapshot_count={rejected_snapshot_count}"),
        ],
        Some(stable_hash_string(&format!("{:?}", inventory))),
    );
    let (pack_series_count, campaign_invocation_status, anchor_invocation_status, later_root) =
        match super::freeze_momentum_historical_evidence_pack_v0(
            std::slice::from_ref(&legacy),
            &policy,
        ) {
            Err(error) => {
                record_execution_stage_v2(
                    &mut trace,
                    JointParticipantExecutionStageV2::EvidencePackConstruction,
                    JointExecutionStageStatusV2::Failed,
                    Some("legacy_pack_construction_failed"),
                    vec![format!("error={error:?}")],
                    None,
                );
                (
                    0,
                    "pack_construction_failed".into(),
                    "not_executed",
                    JointMomentumRootCauseV2::PackInvariantFailure,
                )
            }
            Ok((_, pack)) => {
                let pack_series_count = pack.series.len();
                let pack_valid = super::verify_momentum_historical_evidence_pack_v0(&pack).is_ok();
                record_execution_stage_v2(
                    &mut trace,
                    JointParticipantExecutionStageV2::EvidencePackConstruction,
                    if pack_series_count == 1 {
                        JointExecutionStageStatusV2::Completed
                    } else {
                        JointExecutionStageStatusV2::Failed
                    },
                    (pack_series_count != 1).then_some("legacy_pack_expected_series_missing"),
                    vec![format!("pack_series_count={pack_series_count}")],
                    Some(pack.digest.clone()),
                );
                record_execution_stage_v2(
                    &mut trace,
                    JointParticipantExecutionStageV2::EvidencePackVerification,
                    if pack_valid {
                        JointExecutionStageStatusV2::Completed
                    } else {
                        JointExecutionStageStatusV2::Failed
                    },
                    (!pack_valid).then_some("legacy_pack_verification_failed"),
                    vec![format!("pack_valid={pack_valid}")],
                    Some(pack.digest.clone()),
                );
                if pack_series_count != 1 || !pack_valid {
                    (
                        pack_series_count,
                        "not_invoked_empty_or_invalid_pack".into(),
                        "not_executed",
                        if pack_series_count == 0 {
                            JointMomentumRootCauseV2::EmptyAcceptedSeries
                        } else {
                            JointMomentumRootCauseV2::PackInvariantFailure
                        },
                    )
                } else {
                    match frozen_mamba3_encoder_from_seed_v0(
                        &campaign.feature_config,
                        campaign.campaign_seed,
                        campaign.backend_preference,
                        campaign.fallback_policy,
                    ) {
                        Err(_) => {
                            record_execution_stage_v2(
                                &mut trace,
                                JointParticipantExecutionStageV2::EncoderConstruction,
                                JointExecutionStageStatusV2::Failed,
                                Some("legacy_encoder_construction_failed"),
                                vec![],
                                None,
                            );
                            (
                                pack_series_count,
                                "not_invoked_encoder_failure".into(),
                                "not_executed",
                                JointMomentumRootCauseV2::CampaignRuntimeFailure,
                            )
                        }
                        Ok(encoder) => {
                            record_execution_stage_v2(
                                &mut trace,
                                JointParticipantExecutionStageV2::EncoderConstruction,
                                JointExecutionStageStatusV2::Completed,
                                None,
                                vec![],
                                Some(encoder.parameter_digest()),
                            );
                            let regime = BtcHistoricalRegimeV0 {
                                regime_id: scope.joint_scope_id.clone(),
                                start_row_index: 0,
                                end_row_index_exclusive: legacy.row_count,
                                start_timestamp_ms: legacy
                                    .actual_start_timestamp_ms
                                    .unwrap_or_default(),
                                end_timestamp_ms: legacy
                                    .actual_end_timestamp_ms
                                    .unwrap_or_default(),
                                row_count: legacy.row_count,
                                source_snapshot_id: legacy.snapshot_id.clone(),
                                usage_class: EvidenceUsageClassV0::DevelopmentEligible,
                                segmentation_config_digest: scope.registration_digest_v1.clone(),
                            };
                            match run_btc_historical_regime_campaigns_v0(
                                &[(regime, pack)],
                                campaign,
                                &encoder,
                            ) {
                                Err(error) => {
                                    record_execution_stage_v2(
                                        &mut trace,
                                        JointParticipantExecutionStageV2::CampaignExecution,
                                        JointExecutionStageStatusV2::Failed,
                                        Some("legacy_campaign_invocation_failed"),
                                        vec![format!("error={error:?}")],
                                        None,
                                    );
                                    (
                                        pack_series_count,
                                        "campaign_error".into(),
                                        "not_executed",
                                        JointMomentumRootCauseV2::CampaignConfigurationFailure,
                                    )
                                }
                                Ok(mut results) => {
                                    let Some(result) = results.pop() else {
                                        record_execution_stage_v2(
                                            &mut trace,
                                            JointParticipantExecutionStageV2::CampaignExecution,
                                            JointExecutionStageStatusV2::Failed,
                                            Some("legacy_campaign_output_missing"),
                                            vec![],
                                            None,
                                        );
                                        return Ok(forensic_report_v2(
                                            scope,
                                            legacy.snapshot_id,
                                            legacy.content_digest,
                                            quality_summary_consistent,
                                            accepted_series_count,
                                            rejected_snapshot_count,
                                            pack_series_count,
                                            "campaign_output_missing".into(),
                                            "not_executed".into(),
                                            trace,
                                            JointMomentumRootCauseV2::CampaignConfigurationFailure,
                                        ));
                                    };
                                    record_execution_stage_v2(
                                        &mut trace,
                                        JointParticipantExecutionStageV2::CampaignExecution,
                                        JointExecutionStageStatusV2::Completed,
                                        None,
                                        vec![format!("report_digest={}", result.report_digest)],
                                        Some(result.report_digest.clone()),
                                    );
                                    match momentum_anchor_scope_v1(
                                        &legacy,
                                        0,
                                        legacy.row_count,
                                        campaign,
                                    ) {
                                        Ok(anchor) => {
                                            record_execution_stage_v2(&mut trace, JointParticipantExecutionStageV2::AnchorMaterialization, JointExecutionStageStatusV2::Completed, None, vec![format!("anchor_count={}", anchor.effective_anchor_count)], Some(anchor.scope_digest_v1));
                                            (pack_series_count, "completed".into(), "completed".into(), JointMomentumRootCauseV2::GenuineCompletedNoUsableValidationSignal)
                                        }
                                        Err(_) => {
                                            record_execution_stage_v2(&mut trace, JointParticipantExecutionStageV2::AnchorMaterialization, JointExecutionStageStatusV2::Failed, Some("legacy_anchor_materialization_failed"), vec![], None);
                                            (pack_series_count, "completed".into(), "anchor_error".into(), JointMomentumRootCauseV2::AnchorAuditFailureAfterCompletedCampaign)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
    trace.execution_health = if !identity_consistent {
        JointParticipantExecutionHealthV2::DerivedSnapshotSemanticFailure
    } else if accepted_series_count == 0 {
        JointParticipantExecutionHealthV2::NoAcceptedSeries
    } else {
        match later_root {
            JointMomentumRootCauseV2::PackInvariantFailure => {
                JointParticipantExecutionHealthV2::PackVerificationFailure
            }
            JointMomentumRootCauseV2::CampaignConfigurationFailure => {
                JointParticipantExecutionHealthV2::CampaignConfigurationFailure
            }
            JointMomentumRootCauseV2::CampaignRuntimeFailure => {
                JointParticipantExecutionHealthV2::EncoderConstructionFailure
            }
            _ => JointParticipantExecutionHealthV2::Completed,
        }
    };
    if trace.execution_health == JointParticipantExecutionHealthV2::Completed {
        trace.model_evidence_outcome =
            JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal;
        trace.operational_shadow_result =
            JointParticipantOperationalShadowResultV2::ShadowAbstainNoSignal;
    }
    finish_execution_trace_v2(&mut trace);
    let root_cause = if !identity_consistent {
        JointMomentumRootCauseV2::DerivedSnapshotIdentityMismatch
    } else if accepted_series_count == 0 {
        JointMomentumRootCauseV2::DerivedEvidenceApprovalMissing
    } else {
        later_root
    };
    Ok(forensic_report_v2(
        scope,
        legacy.snapshot_id,
        legacy.content_digest,
        quality_summary_consistent,
        accepted_series_count,
        rejected_snapshot_count,
        pack_series_count,
        campaign_invocation_status,
        anchor_invocation_status.into(),
        trace,
        root_cause,
    ))
}

fn forensic_report_v2(
    scope: &JointCanonicalHistoricalScopeV1,
    derived_snapshot_id: String,
    derived_snapshot_digest: String,
    quality_summary_consistent: bool,
    accepted_series_count: usize,
    rejected_snapshot_count: usize,
    pack_series_count: usize,
    campaign_invocation_status: String,
    anchor_invocation_status: String,
    execution_trace: JointParticipantExecutionTraceV2,
    root_cause: JointMomentumRootCauseV2,
) -> JointMomentumForensicReportV2 {
    let forensic_digest_v2 = joint_v2_digest(&[
        scope.joint_scope_id.clone(),
        derived_snapshot_id.clone(),
        derived_snapshot_digest.clone(),
        quality_summary_consistent.to_string(),
        accepted_series_count.to_string(),
        rejected_snapshot_count.to_string(),
        pack_series_count.to_string(),
        campaign_invocation_status.clone(),
        anchor_invocation_status.clone(),
        format!("{:?}", root_cause),
        execution_trace.trace_digest_v2.clone(),
    ]);
    JointMomentumForensicReportV2 {
        report_version: "joint-momentum-failure-forensic-v2".into(),
        joint_scope_id: scope.joint_scope_id.clone(),
        derived_snapshot_id,
        derived_snapshot_digest,
        quality_summary_consistent,
        accepted_series_count,
        rejected_snapshot_count,
        pack_series_count,
        campaign_invocation_status,
        anchor_invocation_status,
        execution_trace,
        root_cause,
        forensic_digest_v2,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeParticipantReplayResultV2 {
    pub joint_scope_id: String,
    pub joint_scope_digest: String,
    pub participant_agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub execution_trace: JointParticipantExecutionTraceV2,
    pub completed_result_digest: Option<String>,
    pub anchor_scope_digest: Option<String>,
    pub anchor_status: JointAnchorAuditStatusV2,
    pub opinion_id: Option<String>,
    pub seal_digest: Option<String>,
    pub sealed_opinion: Option<(LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1)>,
    pub result_digest_v2: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeReplayResultV2 {
    pub replay_version: String,
    pub registration_digest_v2: String,
    pub joint_scope_id: String,
    pub joint_scope_digest: String,
    pub derived_snapshot_id: String,
    pub derivation_digest_v2: String,
    pub evidence_policy_digest_v2: String,
    pub momentum: JointScopeParticipantReplayResultV2,
    pub risk: JointScopeParticipantReplayResultV2,
    pub pair_eligible: bool,
    pub result_digest_v2: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeReplayAggregateV2 {
    pub aggregate_version: String,
    pub registration_digest_v2: String,
    pub completed_pair_count: usize,
    pub technical_failure_scope_count: usize,
    pub both_abstained_count: usize,
    pub momentum_abstained_count: usize,
    pub risk_abstained_count: usize,
    pub tension_count: usize,
    pub orthogonal_count: usize,
    pub incomparable_count: usize,
    pub relationship_count: usize,
    pub deliberation_count: usize,
    pub transcript_digests: Vec<String>,
    pub relationships: Vec<JointScopeRelationshipV2>,
    pub deliberations: Vec<JointScopeDeliberationV2>,
    pub full_aggregate_composed: bool,
    pub aggregate_digest_v2: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeReplayLedgerV2 {
    pub ledger_version: String,
    pub registration_digest_v2: String,
    pub participant_result_digests: Vec<String>,
    pub deliberation_transcript_digests: Vec<String>,
    pub ledger_digest_v2: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointScopeRelationshipV2 {
    BothAbstained,
    MomentumAbstained,
    RiskAbstained,
    Tension,
    Orthogonal,
    Incomparable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeDeliberationV2 {
    pub deliberation_version: String,
    pub joint_scope_id: String,
    pub mapping_pair_digest_v1: String,
    pub relationship: JointScopeRelationshipV2,
    pub round_count: usize,
    pub retrospective_only: bool,
    pub chair_observed: bool,
    pub vote_created: bool,
    pub reward_created: bool,
    pub penalty_created: bool,
    pub execution_created: bool,
    pub transcript_digest_v2: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumClosureInvariantV3 {
    RegimeIdentity,
    RegimeRowCount,
    CampaignWindowCount,
    ExecutionHealth,
    DiagnosticCompleteness,
    ModelEvidenceOutcome,
    OperationalShadowResult,
    NoSignalWindowCount,
    SelectedCheckpointCount,
    SelectedCheckpointRange,
    TestSealCount,
    SupportEnvelopeCount,
    SupportApplicabilityCount,
    ValidationSupportCounts,
    TestSupportCounts,
    SupportCountBounds,
    DominantSupportOutcome,
    FirstBreachMetric,
    TemporalRootCause,
    WarmStartStatus,
    AbstentionCount,
    AcceptedVersionCount,
    AcceptedVersionBound,
    NoSignalCheckpointExclusivity,
    ReasonCodeConsistency,
    ExecutionTraceConsistency,
    ExecutionTraceStageCount,
    ReportDigest,
    ReportDigestPresent,
    ValidatorContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumClosureInvariantResultV3 {
    pub invariant: MomentumClosureInvariantV3,
    pub passed: bool,
    pub expected_semantic_value: String,
    pub actual_semantic_value: String,
    pub reason_code: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumClosureFailureClassV3 {
    NoFailure,
    BuilderFieldMismatch,
    WrapperRegimeReferenceMismatch,
    VerdictMappingMismatch,
    SupportInvariantMismatch,
    DiagnosticInvariantMismatch,
    ExecutionTraceMismatch,
    ReportDigestMismatch,
    StaleValidatorContract,
    MultipleMismatches,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumPreClosureEvidenceV3 {
    pub campaign_report_digest: String,
    pub campaign_window_count: usize,
    pub final_verdict: String,
    pub no_signal_window_count: usize,
    pub selected_checkpoint_count: usize,
    pub support_counts: Vec<usize>,
    pub encoder_digest: String,
    pub pack_digest: String,
    pub derived_snapshot_digest: String,
    pub preclosure_digest_v3: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumClosedResultContractAuditV3 {
    pub audit_version: String,
    pub joint_scope_id: String,
    pub open_result_digest: String,
    pub closed_result_digest: String,
    pub regime_reference_digest: String,
    pub preclosure: MomentumPreClosureEvidenceV3,
    pub invariant_results: Vec<MomentumClosureInvariantResultV3>,
    pub first_failed_invariant: Option<MomentumClosureInvariantV3>,
    pub validator_error: Option<String>,
    pub failure_class: MomentumClosureFailureClassV3,
    pub all_invariants_pass: bool,
    pub audit_digest_v3: String,
}

fn joint_v3_digest(parts: &[String]) -> String {
    let mut bytes = Vec::new();
    strv(&mut bytes, "joint-canonical-scope-replay-v3");
    strings(&mut bytes, parts);
    stable_hash_string(&hex(&bytes))
}

fn push_closure_invariant_v3(
    results: &mut Vec<MomentumClosureInvariantResultV3>,
    invariant: MomentumClosureInvariantV3,
    expected_semantic_value: String,
    actual_semantic_value: String,
) {
    let passed = expected_semantic_value == actual_semantic_value;
    results.push(MomentumClosureInvariantResultV3 {
        invariant,
        passed,
        expected_semantic_value,
        actual_semantic_value,
        reason_code: if passed {
            "closure_invariant_passed".into()
        } else {
            format!("closure_invariant_failed_{invariant:?}")
        },
    });
}

fn closure_model_evidence_outcome_v3(
    result: &super::BtcTemporalRegimeEvidenceResultV0,
) -> super::RegimeModelEvidenceOutcomeV0 {
    match result.final_verdict {
        super::SupportGatedMomentumSeriesVerdictV0::NoUsableValidationSignal => {
            super::RegimeModelEvidenceOutcomeV0::NoUsableValidationSignal
        }
        super::SupportGatedMomentumSeriesVerdictV0::TemporalOutOfSupportAbstention => {
            super::RegimeModelEvidenceOutcomeV0::ValidationSignalButOutOfSupport
        }
        super::SupportGatedMomentumSeriesVerdictV0::FrozenRepresentationShiftRisk => {
            super::RegimeModelEvidenceOutcomeV0::FrozenRepresentationShiftRisk
        }
        super::SupportGatedMomentumSeriesVerdictV0::WarmStartLockInRisk => {
            super::RegimeModelEvidenceOutcomeV0::WarmStartLockInRisk
        }
        super::SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalButLinearStrongerOnThisSeries => {
            super::RegimeModelEvidenceOutcomeV0::LinearBaselineStronger
        }
        super::SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalAndMambaHelpedOnThisSeries => {
            super::RegimeModelEvidenceOutcomeV0::InSupportUsableSignal
        }
        super::SupportGatedMomentumSeriesVerdictV0::InSupportMixedEvidence => {
            super::RegimeModelEvidenceOutcomeV0::MixedEvidence
        }
        super::SupportGatedMomentumSeriesVerdictV0::InsufficientEvidence
        | super::SupportGatedMomentumSeriesVerdictV0::CampaignFailed => {
            super::RegimeModelEvidenceOutcomeV0::InsufficientEvidence
        }
    }
}

fn closure_operational_shadow_result_v3(
    result: &super::BtcTemporalRegimeEvidenceResultV0,
) -> super::RegimeOperationalShadowResultV0 {
    if result.selected_checkpoint_windows == 0 {
        super::RegimeOperationalShadowResultV0::ShadowAbstainNoSignal
    } else if result.support_unavailable_windows > 0 {
        super::RegimeOperationalShadowResultV0::ShadowAbstainSupportUnavailable
    } else if result.out_of_support_windows > 0 {
        super::RegimeOperationalShadowResultV0::ShadowAbstainOutOfSupport
    } else if result.in_support_windows > 0 {
        super::RegimeOperationalShadowResultV0::ShadowPredictionResearchOnly
    } else {
        super::RegimeOperationalShadowResultV0::ShadowAbstainInsufficientEvidence
    }
}

fn closure_dominant_support_outcome_v3(
    result: &super::BtcTemporalRegimeEvidenceResultV0,
) -> super::RegimeDominantSupportOutcomeV0 {
    if result.selected_checkpoint_windows == 0 {
        super::RegimeDominantSupportOutcomeV0::NoUsableValidationSignal
    } else if result.validation_in_support_windows > 0
        && (result.validation_out_of_support_windows > 0 || result.out_of_support_windows > 0)
    {
        super::RegimeDominantSupportOutcomeV0::Mixed
    } else if result.validation_out_of_support_windows > 0 {
        super::RegimeDominantSupportOutcomeV0::ValidationOutOfSupport
    } else if result.out_of_support_windows > 0 {
        super::RegimeDominantSupportOutcomeV0::TestOutOfSupport
    } else if result.validation_gate_unavailable_windows > 0 {
        super::RegimeDominantSupportOutcomeV0::SupportGateUnavailable
    } else if result.validation_insufficient_windows > 0 {
        super::RegimeDominantSupportOutcomeV0::InsufficientSupportEvidence
    } else if result.validation_in_support_windows > 0 {
        super::RegimeDominantSupportOutcomeV0::ValidationInSupport
    } else {
        super::RegimeDominantSupportOutcomeV0::Mixed
    }
}

fn closure_expected_trace_v3(
    result: &super::BtcTemporalRegimeEvidenceResultV0,
    operational_shadow_result: super::RegimeOperationalShadowResultV0,
) -> (Vec<super::RegimeExecutionStageResultV0>, String) {
    let no_signal = result.selected_checkpoint_windows == 0;
    let support_abstained = !no_signal
        && operational_shadow_result
            != super::RegimeOperationalShadowResultV0::ShadowPredictionResearchOnly;
    let stages = [
        super::RegimeExecutionStageV0::PackLoad,
        super::RegimeExecutionStageV0::PackDigestVerification,
        super::RegimeExecutionStageV0::RowChronologyVerification,
        super::RegimeExecutionStageV0::CampaignConfiguration,
        super::RegimeExecutionStageV0::FeatureExtraction,
        super::RegimeExecutionStageV0::TrainOnlyNormalization,
        super::RegimeExecutionStageV0::SequenceConstruction,
        super::RegimeExecutionStageV0::WindowConstruction,
        super::RegimeExecutionStageV0::CandidateRegistration,
        super::RegimeExecutionStageV0::CandidateTraining,
        super::RegimeExecutionStageV0::CheckpointTrajectory,
        super::RegimeExecutionStageV0::ValidationSignalGate,
        super::RegimeExecutionStageV0::CheckpointSelection,
        super::RegimeExecutionStageV0::TestSealDecision,
        super::RegimeExecutionStageV0::TestEvaluation,
        super::RegimeExecutionStageV0::TemporalSupportGate,
        super::RegimeExecutionStageV0::TemporalShiftDiagnostics,
        super::RegimeExecutionStageV0::WarmColdDiagnostics,
        super::RegimeExecutionStageV0::OperationalShadowResult,
        super::RegimeExecutionStageV0::ModelVersionConstruction,
        super::RegimeExecutionStageV0::RegimeReportConstruction,
        super::RegimeExecutionStageV0::RegimeReportDigest,
    ]
    .into_iter()
    .map(|stage| {
        let status = match stage {
            super::RegimeExecutionStageV0::ValidationSignalGate if no_signal => {
                super::RegimeExecutionStageStatusV0::CompletedNoSignal
            }
            super::RegimeExecutionStageV0::CheckpointSelection
            | super::RegimeExecutionStageV0::TestEvaluation
            | super::RegimeExecutionStageV0::TemporalSupportGate
            | super::RegimeExecutionStageV0::TemporalShiftDiagnostics
            | super::RegimeExecutionStageV0::ModelVersionConstruction
                if no_signal =>
            {
                super::RegimeExecutionStageStatusV0::NotApplicable
            }
            super::RegimeExecutionStageV0::TemporalSupportGate
            | super::RegimeExecutionStageV0::OperationalShadowResult
                if support_abstained =>
            {
                super::RegimeExecutionStageStatusV0::CompletedAbstained
            }
            super::RegimeExecutionStageV0::ModelVersionConstruction if support_abstained => {
                super::RegimeExecutionStageStatusV0::NotApplicable
            }
            _ => super::RegimeExecutionStageStatusV0::Completed,
        };
        super::RegimeExecutionStageResultV0 {
            stage,
            status,
            reason_codes: Vec::new(),
        }
    })
    .collect::<Vec<_>>();
    let digest = stable_hash_string(&format!(
        "{}:{:?}:{}",
        result.regime_id,
        super::RegimeExecutionHealthV0::Completed,
        stages
            .iter()
            .map(|stage| format!("{:?}:{:?}", stage.stage, stage.status))
            .collect::<Vec<_>>()
            .join(":"),
    ));
    (stages, digest)
}

fn classify_closure_failure_v3(
    failed: &[MomentumClosureInvariantV3],
) -> MomentumClosureFailureClassV3 {
    let semantic_failed = failed
        .iter()
        .copied()
        .filter(|invariant| *invariant != MomentumClosureInvariantV3::ValidatorContract)
        .collect::<Vec<_>>();
    if semantic_failed.is_empty() {
        return if failed.is_empty() {
            MomentumClosureFailureClassV3::NoFailure
        } else {
            MomentumClosureFailureClassV3::StaleValidatorContract
        };
    }
    if semantic_failed.len() > 1 {
        return MomentumClosureFailureClassV3::MultipleMismatches;
    }
    match semantic_failed[0] {
        MomentumClosureInvariantV3::RegimeIdentity | MomentumClosureInvariantV3::RegimeRowCount => {
            MomentumClosureFailureClassV3::WrapperRegimeReferenceMismatch
        }
        MomentumClosureInvariantV3::ModelEvidenceOutcome
        | MomentumClosureInvariantV3::OperationalShadowResult => {
            MomentumClosureFailureClassV3::VerdictMappingMismatch
        }
        MomentumClosureInvariantV3::SupportEnvelopeCount
        | MomentumClosureInvariantV3::SupportApplicabilityCount
        | MomentumClosureInvariantV3::ValidationSupportCounts
        | MomentumClosureInvariantV3::TestSupportCounts
        | MomentumClosureInvariantV3::SupportCountBounds
        | MomentumClosureInvariantV3::DominantSupportOutcome => {
            MomentumClosureFailureClassV3::SupportInvariantMismatch
        }
        MomentumClosureInvariantV3::DiagnosticCompleteness
        | MomentumClosureInvariantV3::FirstBreachMetric
        | MomentumClosureInvariantV3::TemporalRootCause
        | MomentumClosureInvariantV3::WarmStartStatus
        | MomentumClosureInvariantV3::AbstentionCount
        | MomentumClosureInvariantV3::ReasonCodeConsistency => {
            MomentumClosureFailureClassV3::DiagnosticInvariantMismatch
        }
        MomentumClosureInvariantV3::ExecutionTraceConsistency => {
            MomentumClosureFailureClassV3::ExecutionTraceMismatch
        }
        MomentumClosureInvariantV3::ExecutionTraceStageCount => {
            MomentumClosureFailureClassV3::ExecutionTraceMismatch
        }
        MomentumClosureInvariantV3::ReportDigest
        | MomentumClosureInvariantV3::ReportDigestPresent => {
            MomentumClosureFailureClassV3::ReportDigestMismatch
        }
        MomentumClosureInvariantV3::ValidatorContract => {
            MomentumClosureFailureClassV3::StaleValidatorContract
        }
        MomentumClosureInvariantV3::CampaignWindowCount
        | MomentumClosureInvariantV3::ExecutionHealth
        | MomentumClosureInvariantV3::NoSignalWindowCount
        | MomentumClosureInvariantV3::SelectedCheckpointCount
        | MomentumClosureInvariantV3::SelectedCheckpointRange
        | MomentumClosureInvariantV3::TestSealCount
        | MomentumClosureInvariantV3::AcceptedVersionCount
        | MomentumClosureInvariantV3::AcceptedVersionBound => {
            MomentumClosureFailureClassV3::BuilderFieldMismatch
        }
        MomentumClosureInvariantV3::NoSignalCheckpointExclusivity => {
            MomentumClosureFailureClassV3::StaleValidatorContract
        }
    }
}

pub fn audit_momentum_closed_result_contract_v3(
    joint_scope_id: &str,
    result: &super::BtcTemporalRegimeEvidenceResultV0,
    regime: &BtcTemporalRegimeRefV0,
    closed: &super::BtcTemporalRegimeClosedResultV0,
    encoder_digest: String,
    pack_digest: String,
    derived_snapshot_digest: String,
) -> MomentumClosedResultContractAuditV3 {
    let diagnostic_completeness = if result.selected_checkpoint_windows == 0 {
        super::RegimeDiagnosticCompletenessV0::PartialNoSelectedCheckpoint
    } else if result.support_unavailable_windows > 0 {
        super::RegimeDiagnosticCompletenessV0::PartialSupportGateUnavailable
    } else {
        super::RegimeDiagnosticCompletenessV0::Complete
    };
    let model_evidence_outcome = closure_model_evidence_outcome_v3(result);
    let operational_shadow_result = closure_operational_shadow_result_v3(result);
    let dominant_support_outcome = closure_dominant_support_outcome_v3(result);
    let first_breach_metric = result
        .support_traces
        .iter()
        .filter_map(|trace| trace.validation.first_breach_metric)
        .map(|metric| format!("{metric:?}"))
        .next();
    let support_envelope_ready_windows = result
        .support_traces
        .iter()
        .filter(|trace| {
            trace.envelope.construction_status == super::SupportEnvelopeConstructionStatusV0::Ready
        })
        .count();
    let support_gate_applicable_windows = result
        .support_traces
        .iter()
        .filter(|trace| {
            trace.validation.gate_applicability
                == super::SupportGateApplicabilityStatusV0::Applicable
        })
        .count();
    let expected_accepted_versions = if operational_shadow_result
        == super::RegimeOperationalShadowResultV0::ShadowPredictionResearchOnly
    {
        result.accepted_predictive_versions
    } else {
        0
    };
    let mut expected_reason_codes = result.reason_codes.clone();
    if result.accepted_predictive_versions > expected_accepted_versions {
        expected_reason_codes.push("accepted_predictive_version_absent_by_policy".into());
    }
    expected_reason_codes.sort();
    expected_reason_codes.dedup();
    let (expected_stages, expected_trace_digest) =
        closure_expected_trace_v3(result, operational_shadow_result);
    let expected_report_digest = stable_hash_string(&format!(
        "{}:{}:{:?}:{:?}:{:?}:{:?}:{}:{}:{}",
        result.report_digest,
        expected_trace_digest,
        diagnostic_completeness,
        model_evidence_outcome,
        operational_shadow_result,
        dominant_support_outcome,
        result.selected_checkpoint_windows,
        result.in_support_windows,
        expected_reason_codes.join(":"),
    ));
    let validator_error = match super::validate_btc_temporal_regime_closed_result_v0(closed) {
        Ok(()) => None,
        Err(error) => Some(format!("{error:?}")),
    };
    let mut invariant_results = Vec::new();
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::SelectedCheckpointRange,
        "true".into(),
        (closed.selected_checkpoint_windows <= closed.campaign_window_count).to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::SupportCountBounds,
        "true".into(),
        (closed.in_support_windows <= closed.selected_checkpoint_windows
            && closed.out_of_support_windows <= closed.selected_checkpoint_windows
            && closed.support_unavailable_windows <= closed.selected_checkpoint_windows)
            .to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::AcceptedVersionBound,
        "true".into(),
        (closed.accepted_predictive_versions <= closed.in_support_windows).to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::NoSignalCheckpointExclusivity,
        "mixed_window_counts_permitted".into(),
        if closed.no_signal_windows > 0
            && closed.selected_checkpoint_windows > 0
            && validator_error.is_some()
        {
            "rejected_by_legacy_validator".into()
        } else {
            "mixed_window_counts_permitted".into()
        },
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::ExecutionTraceStageCount,
        "22".into(),
        closed.execution_trace.stages.len().to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::ReportDigestPresent,
        "true".into(),
        (!closed.report_digest.is_empty()).to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::RegimeIdentity,
        format!(
            "{}:{}:{}:{}",
            regime.regime_id, regime.chronological_rank, regime.range_digest, regime.pack_digest
        ),
        format!(
            "{}:{}:{}:{}",
            closed.regime.regime_id,
            closed.regime.chronological_rank,
            closed.regime.range_digest,
            closed.regime.pack_digest
        ),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::RegimeRowCount,
        regime.row_count.to_string(),
        closed.regime.row_count.to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::CampaignWindowCount,
        result.campaign_windows.to_string(),
        closed.campaign_window_count.to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::ExecutionHealth,
        format!("{:?}", super::RegimeExecutionHealthV0::Completed),
        format!("{:?}", closed.execution_health),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::DiagnosticCompleteness,
        format!("{diagnostic_completeness:?}"),
        format!("{:?}", closed.diagnostic_completeness),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::ModelEvidenceOutcome,
        format!("{model_evidence_outcome:?}"),
        format!("{:?}", closed.model_evidence_outcome),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::OperationalShadowResult,
        format!("{operational_shadow_result:?}"),
        format!("{:?}", closed.operational_shadow_result),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::NoSignalWindowCount,
        result.no_signal_windows.to_string(),
        closed.no_signal_windows.to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::SelectedCheckpointCount,
        result.selected_checkpoint_windows.to_string(),
        closed.selected_checkpoint_windows.to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::TestSealCount,
        result
            .campaign_windows
            .saturating_sub(result.selected_checkpoint_windows)
            .to_string(),
        closed.test_sealed_windows.to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::SupportEnvelopeCount,
        support_envelope_ready_windows.to_string(),
        closed.support_envelope_ready_windows.to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::SupportApplicabilityCount,
        support_gate_applicable_windows.to_string(),
        closed.support_gate_applicable_windows.to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::ValidationSupportCounts,
        format!(
            "{}:{}:{}:{}",
            result.validation_in_support_windows,
            result.validation_out_of_support_windows,
            result.validation_insufficient_windows,
            result.validation_gate_unavailable_windows
        ),
        format!(
            "{}:{}:{}:{}",
            closed.validation_in_support_windows,
            closed.validation_out_of_support_windows,
            closed.support_insufficient_windows,
            closed.support_gate_unavailable_windows
        ),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::TestSupportCounts,
        format!(
            "{}:{}",
            result.in_support_windows, result.out_of_support_windows
        ),
        format!(
            "{}:{}",
            closed.test_in_support_windows, closed.test_out_of_support_windows
        ),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::DominantSupportOutcome,
        format!("{dominant_support_outcome:?}"),
        format!("{:?}", closed.dominant_support_outcome),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::FirstBreachMetric,
        format!("{first_breach_metric:?}"),
        format!("{:?}", closed.first_breach_metric),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::TemporalRootCause,
        format!(
            "{:?}:{:?}",
            (result.selected_checkpoint_windows > 0).then_some(result.earliest_shift_stage),
            (result.selected_checkpoint_windows > 0).then_some(result.temporal_root_cause)
        ),
        format!(
            "{:?}:{:?}",
            closed.earliest_shift_stage, closed.temporal_root_cause
        ),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::WarmStartStatus,
        format!("{:?}", result.warm_start_status),
        format!("{:?}", closed.warm_start_status),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::AbstentionCount,
        result.abstention_count.to_string(),
        closed.abstention_count.to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::AcceptedVersionCount,
        expected_accepted_versions.to_string(),
        closed.accepted_predictive_versions.to_string(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::ReasonCodeConsistency,
        expected_reason_codes.join(":"),
        closed.reason_codes.join(":"),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::ExecutionTraceConsistency,
        format!(
            "{}:{:?}:{:?}:{}",
            result.regime_id,
            super::RegimeExecutionHealthV0::Completed,
            expected_stages,
            expected_trace_digest
        ),
        format!(
            "{}:{:?}:{:?}:{}",
            closed.execution_trace.regime_id,
            closed.execution_trace.execution_health,
            closed.execution_trace.stages,
            closed.execution_trace.trace_digest
        ),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::ReportDigest,
        expected_report_digest,
        closed.report_digest.clone(),
    );
    push_closure_invariant_v3(
        &mut invariant_results,
        MomentumClosureInvariantV3::ValidatorContract,
        "Valid".into(),
        validator_error.clone().unwrap_or_else(|| "Valid".into()),
    );
    let failed = invariant_results
        .iter()
        .filter(|result| !result.passed)
        .map(|result| result.invariant)
        .collect::<Vec<_>>();
    let first_failed_invariant = failed.first().copied();
    let failure_class = classify_closure_failure_v3(&failed);
    let preclosure = MomentumPreClosureEvidenceV3 {
        campaign_report_digest: result.report_digest.clone(),
        campaign_window_count: result.campaign_windows,
        final_verdict: format!("{:?}", result.final_verdict),
        no_signal_window_count: result.no_signal_windows,
        selected_checkpoint_count: result.selected_checkpoint_windows,
        support_counts: vec![
            result.in_support_windows,
            result.out_of_support_windows,
            result.support_unavailable_windows,
            result.validation_in_support_windows,
            result.validation_out_of_support_windows,
            result.validation_insufficient_windows,
            result.validation_gate_unavailable_windows,
        ],
        encoder_digest,
        pack_digest,
        derived_snapshot_digest,
        preclosure_digest_v3: String::new(),
    };
    let mut audit = MomentumClosedResultContractAuditV3 {
        audit_version: "momentum-closed-result-contract-audit-v3".into(),
        joint_scope_id: joint_scope_id.into(),
        open_result_digest: result.report_digest.clone(),
        closed_result_digest: closed.report_digest.clone(),
        regime_reference_digest: stable_hash_string(&format!("{regime:?}")),
        preclosure,
        invariant_results,
        first_failed_invariant,
        validator_error,
        failure_class,
        all_invariants_pass: failed.is_empty(),
        audit_digest_v3: String::new(),
    };
    audit.preclosure.preclosure_digest_v3 = joint_v3_digest(&[
        audit.preclosure.campaign_report_digest.clone(),
        audit.preclosure.campaign_window_count.to_string(),
        audit.preclosure.final_verdict.clone(),
        audit.preclosure.no_signal_window_count.to_string(),
        audit.preclosure.selected_checkpoint_count.to_string(),
        audit
            .preclosure
            .support_counts
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(":"),
        audit.preclosure.encoder_digest.clone(),
        audit.preclosure.pack_digest.clone(),
        audit.preclosure.derived_snapshot_digest.clone(),
    ]);
    audit.audit_digest_v3 = joint_v3_digest(&[
        audit.audit_version.clone(),
        audit.joint_scope_id.clone(),
        audit.open_result_digest.clone(),
        audit.closed_result_digest.clone(),
        audit.regime_reference_digest.clone(),
        audit.preclosure.preclosure_digest_v3.clone(),
        audit
            .invariant_results
            .iter()
            .map(|value| {
                format!(
                    "{:?}:{}:{}:{}:{}",
                    value.invariant,
                    value.passed,
                    value.expected_semantic_value,
                    value.actual_semantic_value,
                    value.reason_code
                )
            })
            .collect::<Vec<_>>()
            .join(":"),
        format!("{:?}", audit.first_failed_invariant),
        audit.validator_error.clone().unwrap_or_default(),
        format!("{:?}", audit.failure_class),
        audit.all_invariants_pass.to_string(),
    ]);
    audit
}

pub fn audit_joint_scope_momentum_closure_v3(
    snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
    registration: &JointCanonicalScopeReplayRegistrationV2,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<MomentumClosedResultContractAuditV3, String> {
    let registered_scopes =
        validate_joint_canonical_scope_registration_v2(snapshot, campaign, registration)?;
    if !registered_scopes
        .iter()
        .any(|registered| registered == scope)
    {
        return Err("joint_v3_closure_audit_scope_not_registered".into());
    }
    let derived = derive_joint_scope_snapshot_v2(snapshot, scope)?;
    let policy = joint_scope_derived_evidence_policy_v2(&derived)?;
    let (_, pack) = joint_scope_momentum_pack_v2(&derived, &policy)?;
    let encoder = frozen_mamba3_encoder_from_seed_v0(
        &campaign.feature_config,
        campaign.campaign_seed,
        campaign.backend_preference,
        campaign.fallback_policy,
    )
    .map_err(|error| format!("joint_v3_closure_audit_encoder_{error:?}"))?;
    let regime = BtcHistoricalRegimeV0 {
        regime_id: scope.joint_scope_id.clone(),
        start_row_index: 0,
        end_row_index_exclusive: derived.derived_snapshot.row_count,
        start_timestamp_ms: derived
            .derived_snapshot
            .actual_start_timestamp_ms
            .ok_or("joint_v3_closure_audit_scope_start_missing")?,
        end_timestamp_ms: derived
            .derived_snapshot
            .actual_end_timestamp_ms
            .ok_or("joint_v3_closure_audit_scope_end_missing")?,
        row_count: derived.derived_snapshot.row_count,
        source_snapshot_id: derived.derived_snapshot.snapshot_id.clone(),
        usage_class: EvidenceUsageClassV0::DevelopmentEligible,
        segmentation_config_digest: registration.registration_digest_v2.clone(),
    };
    let mut results = run_btc_historical_regime_campaigns_v0(
        &[(regime.clone(), pack.clone())],
        campaign,
        &encoder,
    )
    .map_err(|error| format!("joint_v3_closure_audit_campaign_{error:?}"))?;
    if results.len() != 1 {
        return Err(format!(
            "joint_v3_closure_audit_campaign_result_count={}",
            results.len()
        ));
    }
    let result = results.remove(0);
    let regime_reference = BtcTemporalRegimeRefV0 {
        regime_id: regime.regime_id,
        chronological_rank: scope.chronological_rank,
        row_count: regime.row_count,
        range_digest: scope.scope_digest_v1.clone(),
        pack_digest: pack.digest.clone(),
    };
    let closed = close_btc_temporal_regime_result_v0(&result, regime_reference.clone());
    Ok(audit_momentum_closed_result_contract_v3(
        &scope.joint_scope_id,
        &result,
        &regime_reference,
        &closed,
        encoder.parameter_digest(),
        pack.digest,
        derived.derived_snapshot.content_digest,
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointCanonicalScopeReplayRegistrationV3 {
    pub registration_version: String,
    pub parent_registration_digest_v2: String,
    pub parent_registration_v2: JointCanonicalScopeReplayRegistrationV2,
    pub joint_scope_ids: Vec<String>,
    pub joint_scope_digests: Vec<String>,
    pub momentum_campaign_config_digest: String,
    pub risk_config_digest: String,
    pub closure_audit_policy_digest: String,
    pub correction_failure_class: MomentumClosureFailureClassV3,
    pub corrected_closure_policy_digest: String,
    pub preclosure_result_digests: Vec<String>,
    pub scope_ranges_unchanged: bool,
    pub participant_configs_unchanged: bool,
    pub preclosure_results_unchanged: bool,
    pub scope0_non_regression_required: bool,
    pub result_dependent_model_changes_forbidden: bool,
    pub authority_policy_digest: String,
    pub registration_digest_v3: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeReplayResultV3 {
    pub replay_version: String,
    pub registration_digest_v3: String,
    pub joint_scope_id: String,
    pub joint_scope_digest: String,
    pub preclosure_digest_v3: String,
    pub closure_audit_digest_v3: String,
    pub parent_result_digest_v2: String,
    pub replay_result_v2: JointScopeReplayResultV2,
    pub result_digest_v3: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeReplayAggregateV3 {
    pub aggregate_version: String,
    pub registration_digest_v3: String,
    pub closure_audit_digests_v3: Vec<String>,
    pub parent_aggregate_digest_v2: String,
    pub replay_aggregate_v2: JointScopeReplayAggregateV2,
    pub aggregate_digest_v3: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JointScopeReplayLedgerV3 {
    pub ledger_version: String,
    pub registration_digest_v3: String,
    pub replay_result_digests_v3: Vec<String>,
    pub deliberation_transcript_digests: Vec<String>,
    pub ledger_digest_v3: String,
}

fn joint_v3_policy_digest(label: &str, parts: &[&str]) -> String {
    let mut values = vec![label.to_string()];
    values.extend(parts.iter().map(|part| (*part).to_string()));
    joint_v3_digest(&values)
}

fn joint_v3_registration_digest(value: &JointCanonicalScopeReplayRegistrationV3) -> String {
    let mut values = vec![
        value.registration_version.clone(),
        value.parent_registration_digest_v2.clone(),
        value.momentum_campaign_config_digest.clone(),
        value.risk_config_digest.clone(),
        value.closure_audit_policy_digest.clone(),
        format!("{:?}", value.correction_failure_class),
        value.corrected_closure_policy_digest.clone(),
        value.authority_policy_digest.clone(),
    ];
    values.extend(value.joint_scope_ids.clone());
    values.extend(value.joint_scope_digests.clone());
    values.extend(value.preclosure_result_digests.clone());
    values.extend([
        value.scope_ranges_unchanged.to_string(),
        value.participant_configs_unchanged.to_string(),
        value.preclosure_results_unchanged.to_string(),
        value.scope0_non_regression_required.to_string(),
        value.result_dependent_model_changes_forbidden.to_string(),
    ]);
    joint_v3_digest(&values)
}

fn v3_correction_class_from_audits(
    audits: &[MomentumClosedResultContractAuditV3],
) -> Result<MomentumClosureFailureClassV3, String> {
    let mixed_window_contracts = audits
        .iter()
        .filter(|audit| {
            audit.all_invariants_pass
                && audit.validator_error.is_none()
                && audit.preclosure.no_signal_window_count > 0
                && audit.preclosure.selected_checkpoint_count > 0
        })
        .count();
    if mixed_window_contracts != 1 {
        return Err("joint_v3_correction_class_unproven".into());
    }
    Ok(MomentumClosureFailureClassV3::StaleValidatorContract)
}

pub fn joint_canonical_scope_registration_v3(
    snapshot: &DataSnapshot,
    campaign: &MomentumLearningCampaignConfigV0,
    audits: &[MomentumClosedResultContractAuditV3],
) -> Result<JointCanonicalScopeReplayRegistrationV3, String> {
    let parent = joint_canonical_scope_registration_v2(snapshot, campaign)?;
    let scopes = validate_joint_canonical_scope_registration_v2(snapshot, campaign, &parent)?;
    if audits.len() != scopes.len()
        || audits.iter().any(|audit| {
            !audit.all_invariants_pass
                || audit.validator_error.is_some()
                || audit.preclosure.preclosure_digest_v3.is_empty()
        })
        || audits
            .iter()
            .map(|audit| audit.joint_scope_id.clone())
            .collect::<Vec<_>>()
            != scopes
                .iter()
                .map(|scope| scope.joint_scope_id.clone())
                .collect::<Vec<_>>()
    {
        return Err("joint_v3_registration_audit_invalid".into());
    }
    let correction_failure_class = v3_correction_class_from_audits(audits)?;
    let mut value = JointCanonicalScopeReplayRegistrationV3 {
        registration_version: "joint-canonical-scope-replay-registration-v3".into(),
        parent_registration_digest_v2: parent.registration_digest_v2.clone(),
        parent_registration_v2: parent.clone(),
        joint_scope_ids: scopes
            .iter()
            .map(|scope| scope.joint_scope_id.clone())
            .collect(),
        joint_scope_digests: scopes
            .iter()
            .map(|scope| scope.scope_digest_v1.clone())
            .collect(),
        momentum_campaign_config_digest: campaign.digest(),
        risk_config_digest: CycleRiskShadowConfigV0::default().digest(),
        closure_audit_policy_digest: joint_v3_policy_digest(
            "momentum-closed-result-contract-audit-policy-v3",
            &[
                "field-level-builder-audit",
                "typed-validator-provenance",
                "deterministic-audit-replay",
                "no-raw-market-or-model-output",
            ],
        ),
        correction_failure_class,
        corrected_closure_policy_digest: joint_v3_policy_digest(
            "momentum-closed-result-validator-policy-v3",
            &[
                "per-window-counts-may-coexist",
                "checkpoint-and-support-bounds-retained",
                "execution-trace-and-report-digest-retained",
                "no-model-or-scope-change",
            ],
        ),
        preclosure_result_digests: audits
            .iter()
            .map(|audit| audit.preclosure.preclosure_digest_v3.clone())
            .collect(),
        scope_ranges_unchanged: true,
        participant_configs_unchanged: true,
        preclosure_results_unchanged: true,
        scope0_non_regression_required: true,
        result_dependent_model_changes_forbidden: true,
        authority_policy_digest: joint_v3_policy_digest(
            "joint-authority-policy-v3",
            &[
                "advisory-only",
                "chair-vote-reward-penalty-promotion-execution-forbidden",
            ],
        ),
        registration_digest_v3: String::new(),
    };
    value.registration_digest_v3 = joint_v3_registration_digest(&value);
    Ok(value)
}

pub fn validate_joint_canonical_scope_registration_v3(
    snapshot: &DataSnapshot,
    campaign: &MomentumLearningCampaignConfigV0,
    registration: &JointCanonicalScopeReplayRegistrationV3,
) -> Result<Vec<JointCanonicalHistoricalScopeV1>, String> {
    if registration.registration_version != "joint-canonical-scope-replay-registration-v3"
        || registration.registration_digest_v3 != joint_v3_registration_digest(registration)
        || !registration.scope_ranges_unchanged
        || !registration.participant_configs_unchanged
        || !registration.preclosure_results_unchanged
        || !registration.scope0_non_regression_required
        || !registration.result_dependent_model_changes_forbidden
        || registration.correction_failure_class == MomentumClosureFailureClassV3::Unknown
        || registration.momentum_campaign_config_digest != campaign.digest()
        || registration.risk_config_digest != CycleRiskShadowConfigV0::default().digest()
    {
        return Err("joint_v3_registration_invalid".into());
    }
    let parent = joint_canonical_scope_registration_v2(snapshot, campaign)?;
    let scopes = validate_joint_canonical_scope_registration_v2(snapshot, campaign, &parent)?;
    if registration.parent_registration_digest_v2 != parent.registration_digest_v2
        || registration.parent_registration_v2 != parent
        || registration.joint_scope_ids
            != scopes
                .iter()
                .map(|scope| scope.joint_scope_id.clone())
                .collect::<Vec<_>>()
        || registration.joint_scope_digests
            != scopes
                .iter()
                .map(|scope| scope.scope_digest_v1.clone())
                .collect::<Vec<_>>()
        || registration.preclosure_result_digests.len() != scopes.len()
    {
        return Err("joint_v3_scope_reuse_mismatch".into());
    }
    Ok(scopes)
}

pub fn replay_joint_scope_results_v3(
    snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
    registration: &JointCanonicalScopeReplayRegistrationV3,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<JointScopeReplayResultV3, String> {
    let scopes = validate_joint_canonical_scope_registration_v3(snapshot, campaign, registration)?;
    let scope_index = scopes
        .iter()
        .position(|registered| registered == scope)
        .ok_or("joint_v3_scope_not_registered")?;
    let parent = joint_canonical_scope_registration_v2(snapshot, campaign)?;
    let audit = audit_joint_scope_momentum_closure_v3(snapshot, scope, &parent, campaign)?;
    if !audit.all_invariants_pass
        || audit.validator_error.is_some()
        || audit.preclosure.preclosure_digest_v3
            != registration.preclosure_result_digests[scope_index]
    {
        return Err("joint_v3_preclosure_freeze_or_closure_invalid".into());
    }
    let replay_result_v2 = replay_joint_scope_results_v2(snapshot, scope, &parent, campaign)?;
    let result_digest_v3 = joint_v3_digest(&[
        registration.registration_digest_v3.clone(),
        scope.joint_scope_id.clone(),
        scope.scope_digest_v1.clone(),
        audit.preclosure.preclosure_digest_v3.clone(),
        audit.audit_digest_v3.clone(),
        replay_result_v2.result_digest_v2.clone(),
    ]);
    Ok(JointScopeReplayResultV3 {
        replay_version: "joint-canonical-scope-replay-v3".into(),
        registration_digest_v3: registration.registration_digest_v3.clone(),
        joint_scope_id: scope.joint_scope_id.clone(),
        joint_scope_digest: scope.scope_digest_v1.clone(),
        preclosure_digest_v3: audit.preclosure.preclosure_digest_v3,
        closure_audit_digest_v3: audit.audit_digest_v3,
        parent_result_digest_v2: replay_result_v2.result_digest_v2.clone(),
        replay_result_v2,
        result_digest_v3,
    })
}

pub fn aggregate_joint_scope_replays_v3(
    registration: &JointCanonicalScopeReplayRegistrationV3,
    results: &[JointScopeReplayResultV3],
) -> Result<(JointScopeReplayAggregateV3, JointScopeReplayLedgerV3), String> {
    if registration.registration_digest_v3 != joint_v3_registration_digest(registration)
        || results.len() != registration.joint_scope_ids.len()
        || results.iter().any(|result| {
            result.registration_digest_v3 != registration.registration_digest_v3
                || !registration
                    .joint_scope_ids
                    .contains(&result.joint_scope_id)
                || !registration
                    .joint_scope_digests
                    .contains(&result.joint_scope_digest)
        })
    {
        return Err("joint_v3_aggregate_input_invalid".into());
    }
    let parent_results = results
        .iter()
        .map(|result| result.replay_result_v2.clone())
        .collect::<Vec<_>>();
    let (replay_aggregate_v2, _) =
        aggregate_joint_scope_replays_v2(&registration.parent_registration_v2, &parent_results)?;
    let closure_audit_digests_v3 = results
        .iter()
        .map(|result| result.closure_audit_digest_v3.clone())
        .collect::<Vec<_>>();
    let mut aggregate = JointScopeReplayAggregateV3 {
        aggregate_version: "joint-scope-relationship-only-aggregate-v3".into(),
        registration_digest_v3: registration.registration_digest_v3.clone(),
        closure_audit_digests_v3,
        parent_aggregate_digest_v2: replay_aggregate_v2.aggregate_digest_v2.clone(),
        replay_aggregate_v2,
        aggregate_digest_v3: String::new(),
    };
    aggregate.aggregate_digest_v3 = joint_v3_digest(&[
        aggregate.aggregate_version.clone(),
        aggregate.registration_digest_v3.clone(),
        aggregate.closure_audit_digests_v3.join(":"),
        aggregate.parent_aggregate_digest_v2.clone(),
    ]);
    let mut replay_result_digests_v3 = results
        .iter()
        .map(|result| result.result_digest_v3.clone())
        .collect::<Vec<_>>();
    replay_result_digests_v3.sort();
    let mut deliberation_transcript_digests =
        aggregate.replay_aggregate_v2.transcript_digests.clone();
    deliberation_transcript_digests.sort();
    let mut ledger = JointScopeReplayLedgerV3 {
        ledger_version: "joint-scope-replay-ledger-v3".into(),
        registration_digest_v3: registration.registration_digest_v3.clone(),
        replay_result_digests_v3,
        deliberation_transcript_digests,
        ledger_digest_v3: String::new(),
    };
    ledger.ledger_digest_v3 = joint_v3_digest(&[
        ledger.ledger_version.clone(),
        ledger.registration_digest_v3.clone(),
        ledger.replay_result_digests_v3.join(":"),
        ledger.deliberation_transcript_digests.join(":"),
    ]);
    validate_joint_scope_replay_ledger_v3(&ledger)?;
    Ok((aggregate, ledger))
}

pub fn validate_joint_scope_replay_ledger_v3(
    ledger: &JointScopeReplayLedgerV3,
) -> Result<(), String> {
    if ledger.ledger_version != "joint-scope-replay-ledger-v3"
        || ledger
            .replay_result_digests_v3
            .windows(2)
            .any(|pair| pair[0] > pair[1])
        || ledger.ledger_digest_v3
            != joint_v3_digest(&[
                ledger.ledger_version.clone(),
                ledger.registration_digest_v3.clone(),
                ledger.replay_result_digests_v3.join(":"),
                ledger.deliberation_transcript_digests.join(":"),
            ])
    {
        return Err("joint_v3_replay_ledger_invalid".into());
    }
    Ok(())
}

fn momentum_model_outcome_v2(
    result: &super::BtcTemporalRegimeEvidenceResultV0,
) -> JointParticipantModelEvidenceOutcomeV2 {
    match result.final_verdict {
        super::SupportGatedMomentumSeriesVerdictV0::NoUsableValidationSignal => {
            JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal
        }
        super::SupportGatedMomentumSeriesVerdictV0::TemporalOutOfSupportAbstention => {
            JointParticipantModelEvidenceOutcomeV2::ValidationSignalOutOfSupport
        }
        super::SupportGatedMomentumSeriesVerdictV0::FrozenRepresentationShiftRisk
        | super::SupportGatedMomentumSeriesVerdictV0::WarmStartLockInRisk => {
            JointParticipantModelEvidenceOutcomeV2::RepresentationShiftRisk
        }
        super::SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalButLinearStrongerOnThisSeries => {
            JointParticipantModelEvidenceOutcomeV2::BaselineStronger
        }
        super::SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalAndMambaHelpedOnThisSeries
        | super::SupportGatedMomentumSeriesVerdictV0::InSupportMixedEvidence => {
            JointParticipantModelEvidenceOutcomeV2::UsableValidationSignal
        }
        super::SupportGatedMomentumSeriesVerdictV0::InsufficientEvidence
        | super::SupportGatedMomentumSeriesVerdictV0::CampaignFailed => {
            JointParticipantModelEvidenceOutcomeV2::InsufficientEvidence
        }
    }
}

fn operational_shadow_result_v2(
    health: JointParticipantExecutionHealthV2,
    outcome: JointParticipantModelEvidenceOutcomeV2,
) -> JointParticipantOperationalShadowResultV2 {
    if health != JointParticipantExecutionHealthV2::Completed {
        return JointParticipantOperationalShadowResultV2::ShadowAbstainTechnicalFailure;
    }
    match outcome {
        JointParticipantModelEvidenceOutcomeV2::UsableValidationSignal => {
            JointParticipantOperationalShadowResultV2::ShadowPredictionResearchOnly
        }
        JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal
        | JointParticipantModelEvidenceOutcomeV2::ProbabilityCollapse => {
            JointParticipantOperationalShadowResultV2::ShadowAbstainNoSignal
        }
        JointParticipantModelEvidenceOutcomeV2::ValidationSignalOutOfSupport
        | JointParticipantModelEvidenceOutcomeV2::RepresentationShiftRisk => {
            JointParticipantOperationalShadowResultV2::ShadowAbstainOutOfSupport
        }
        JointParticipantModelEvidenceOutcomeV2::InsufficientEvidence
        | JointParticipantModelEvidenceOutcomeV2::BaselineStronger => {
            JointParticipantOperationalShadowResultV2::ShadowAbstainInsufficientEvidence
        }
        JointParticipantModelEvidenceOutcomeV2::NotEvaluatedTechnicalFailure => {
            JointParticipantOperationalShadowResultV2::ShadowAbstainTechnicalFailure
        }
    }
}

pub fn joint_scope_momentum_pack_v2(
    derived: &JointScopeDerivedSnapshotV2,
    policy: &JointScopeDerivedEvidencePolicyV2,
) -> Result<
    (
        super::HistoricalSnapshotInventoryV0,
        super::MomentumHistoricalEvidencePackV0,
    ),
    String,
> {
    if !policy.exact_child_authorized
        || policy.wildcard_authorization
        || policy.derived_snapshot_id != derived.derived_snapshot.snapshot_id
        || policy.derivation_proof_digest != derived.derivation_proof.proof_digest_v2
    {
        return Err("joint_exact_child_evidence_policy_invalid".into());
    }
    let mut historical_policy = HistoricalEvidencePolicyV0::default();
    historical_policy
        .owner_sanitized_snapshot_ids
        .insert(derived.derived_snapshot.snapshot_id.clone());
    let inventory = super::inventory_historical_snapshots_v0(
        std::slice::from_ref(&derived.derived_snapshot),
        &historical_policy,
    )
    .map_err(|error| format!("joint_inventory_error_{error:?}"))?;
    if inventory.accepted_series.len() != 1
        || inventory
            .rejected_snapshots
            .iter()
            .any(|rejected| rejected.snapshot_id == derived.derived_snapshot.snapshot_id)
    {
        return Err("joint_expected_child_inventory_rejected".into());
    }
    let (_, pack) = super::freeze_momentum_historical_evidence_pack_v0(
        std::slice::from_ref(&derived.derived_snapshot),
        &historical_policy,
    )
    .map_err(|error| format!("joint_pack_construction_error_{error:?}"))?;
    if pack.series.len() != 1
        || pack.created_from_snapshot_ids != vec![derived.derived_snapshot.snapshot_id.clone()]
        || pack.series[0].symbol != derived.derived_snapshot.normalized_dataset.symbol
        || pack.series[0].snapshots.len() != 1
        || pack.series[0].snapshots[0].row_count != derived.derived_snapshot.row_count
    {
        return Err("joint_pack_invariant_invalid".into());
    }
    super::verify_momentum_historical_evidence_pack_v0(&pack)
        .map_err(|error| format!("joint_pack_verification_error_{error:?}"))?;
    Ok((inventory, pack))
}

fn participant_result_digest_v2(value: &JointScopeParticipantReplayResultV2) -> String {
    joint_v2_digest(&[
        value.joint_scope_id.clone(),
        value.joint_scope_digest.clone(),
        value.participant_agent_id.clone(),
        format!("{:?}", value.objective),
        value.execution_trace.trace_digest_v2.clone(),
        value.completed_result_digest.clone().unwrap_or_default(),
        value.anchor_scope_digest.clone().unwrap_or_default(),
        format!("{:?}", value.anchor_status),
        value.opinion_id.clone().unwrap_or_default(),
        value.seal_digest.clone().unwrap_or_default(),
    ])
}

fn make_joint_scope_opinion_v2(
    agent_id: String,
    objective: LearnedAgentObjectiveV0,
    source_snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
    result_kind: SourceResultKindV1,
    result_digest: String,
    checkpoint_digest: String,
    pack_digest: String,
    model_version: Option<String>,
    model_artifact_digest: String,
    anchors: &AgentEffectiveAnchorScopeV1,
    forecast_scope_digest: String,
    reason_code: &str,
) -> Result<(LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1), String> {
    let mut source = LearnedAgentSourceResultReferenceV1 {
        agent_id,
        objective,
        source_snapshot_id: source_snapshot.snapshot_id.clone(),
        source_snapshot_digest: source_snapshot.content_digest.clone(),
        source_result_kind: result_kind,
        source_result_digest_v1: result_digest.clone(),
        source_checkpoint_digest_v1: checkpoint_digest,
        source_frozen_pack_digest: pack_digest,
        source_model_version_id: model_version,
        source_model_artifact_digest: model_artifact_digest,
        canonical_raw_scope_digest_v1: scope.canonical_raw_scope.scope_digest_v1.clone(),
        canonical_raw_row_identity_digests_v1: scope
            .canonical_raw_scope
            .row_identity_digests
            .clone(),
        information_cutoff_timestamp: scope.information_cutoff_timestamp,
        effective_anchor_scope_digest_v1: anchors.scope_digest_v1.clone(),
        effective_anchor_digests_v1: anchors.all_anchor_digests.clone(),
        forecast_scope_digest_v1: forecast_scope_digest,
        reference_digest_v1: String::new(),
    };
    source.reference_digest_v1 = source_reference_digest_v1(&source);
    let mut membership = SourceResultMembershipProofV1 {
        result_digest_v1: result_digest,
        parent_report_digest: source_snapshot.content_digest.clone(),
        immutable_member: true,
        snapshot_matches: true,
        pack_matches: true,
        scope_matches: true,
        anchors_match: !anchors.all_anchor_digests.is_empty(),
        objective_matches: true,
        agent_matches: true,
        all_invariants_pass: !anchors.all_anchor_digests.is_empty(),
        proof_digest_v1: String::new(),
    };
    membership.proof_digest_v1 = strings_digest_v1(
        "joint-v2-membership-proof",
        &[
            membership.result_digest_v1.clone(),
            membership.parent_report_digest.clone(),
            source.reference_digest_v1.clone(),
        ],
    );
    let registration = SourceBoundOpinionProtocolRegistrationV1::pre_registered();
    let mut opinion = create_joint_scope_source_bound_opinion_v1(
        source,
        &membership,
        scope.information_cutoff_timestamp,
        &scope.joint_scope_id,
        reason_code,
        &registration,
    )?;
    let seal = source_bound_seal_v1(
        &opinion.opinion_id,
        &opinion.opinion_digest_v1,
        &opinion.source_result,
        &registration,
        &opinion.authority,
    )?;
    opinion.sealed = true;
    Ok((opinion, seal))
}

pub fn replay_joint_scope_results_v2(
    snapshot: &DataSnapshot,
    scope: &JointCanonicalHistoricalScopeV1,
    registration: &JointCanonicalScopeReplayRegistrationV2,
    campaign: &MomentumLearningCampaignConfigV0,
) -> Result<JointScopeReplayResultV2, String> {
    let registered_scopes =
        validate_joint_canonical_scope_registration_v2(snapshot, campaign, registration)?;
    if !registered_scopes
        .iter()
        .any(|registered| registered == scope)
    {
        return Err("joint_v2_scope_not_registered".into());
    }
    let derived = derive_joint_scope_snapshot_v2(snapshot, scope)?;
    let evidence_policy = joint_scope_derived_evidence_policy_v2(&derived)?;

    let mut momentum_trace = new_execution_trace_v2(
        scope,
        campaign.agent_id.clone(),
        LearnedAgentObjectiveV0::DirectionalMomentum,
    );
    for (stage, digest) in [
        (
            JointParticipantExecutionStageV2::ParentSnapshotVerification,
            snapshot.content_digest.clone(),
        ),
        (
            JointParticipantExecutionStageV2::JointScopeVerification,
            scope.scope_digest_v1.clone(),
        ),
        (
            JointParticipantExecutionStageV2::DerivedSnapshotConstruction,
            derived.derivation_digest_v2.clone(),
        ),
        (
            JointParticipantExecutionStageV2::DerivedSnapshotIdentity,
            derived.derived_snapshot.snapshot_id.clone(),
        ),
        (
            JointParticipantExecutionStageV2::DerivedSnapshotSemanticVerification,
            derived.derivation_proof.proof_digest_v2.clone(),
        ),
        (
            JointParticipantExecutionStageV2::EvidencePolicyConstruction,
            evidence_policy.policy_digest_v2.clone(),
        ),
    ] {
        record_execution_stage_v2(
            &mut momentum_trace,
            stage,
            JointExecutionStageStatusV2::Completed,
            None,
            vec![],
            Some(digest),
        );
    }

    let mut historical_policy = HistoricalEvidencePolicyV0::default();
    historical_policy
        .owner_sanitized_snapshot_ids
        .insert(derived.derived_snapshot.snapshot_id.clone());
    let inventory = super::inventory_historical_snapshots_v0(
        std::slice::from_ref(&derived.derived_snapshot),
        &historical_policy,
    )
    .map_err(|error| format!("joint_v2_inventory_error_{error:?}"))?;
    let accepted_child = inventory.accepted_series.len() == 1
        && inventory
            .rejected_snapshots
            .iter()
            .all(|rejected| rejected.snapshot_id != derived.derived_snapshot.snapshot_id);
    record_execution_stage_v2(
        &mut momentum_trace,
        JointParticipantExecutionStageV2::SnapshotInventoryClassification,
        if accepted_child {
            JointExecutionStageStatusV2::Completed
        } else {
            JointExecutionStageStatusV2::Failed
        },
        (!accepted_child).then_some("joint_expected_child_inventory_rejected"),
        vec![
            format!("accepted_series_count={}", inventory.accepted_series.len()),
            format!(
                "rejected_snapshot_count={}",
                inventory.rejected_snapshots.len()
            ),
        ],
        Some(stable_hash_string(&format!("{:?}", inventory))),
    );

    let mut momentum_result_digest = None;
    let mut momentum_anchor_digest = None;
    let mut momentum_anchor_status = JointAnchorAuditStatusV2::TechnicalFailure;
    let mut momentum_opinion = None;
    let mut momentum_health = if accepted_child {
        JointParticipantExecutionHealthV2::TechnicalFailure
    } else if inventory.accepted_series.is_empty() {
        JointParticipantExecutionHealthV2::NoAcceptedSeries
    } else {
        JointParticipantExecutionHealthV2::InventoryRejected
    };
    let mut momentum_outcome = JointParticipantModelEvidenceOutcomeV2::NotEvaluatedTechnicalFailure;

    if accepted_child {
        match super::freeze_momentum_historical_evidence_pack_v0(
            std::slice::from_ref(&derived.derived_snapshot),
            &historical_policy,
        ) {
            Err(error) => {
                record_execution_stage_v2(
                    &mut momentum_trace,
                    JointParticipantExecutionStageV2::EvidencePackConstruction,
                    JointExecutionStageStatusV2::Failed,
                    Some("joint_v2_pack_construction_failed"),
                    vec![format!("error={error:?}")],
                    None,
                );
                momentum_health = JointParticipantExecutionHealthV2::PackConstructionFailure;
            }
            Ok((_, pack)) => {
                let pack_valid = pack.series.len() == 1
                    && pack.created_from_snapshot_ids
                        == vec![derived.derived_snapshot.snapshot_id.clone()]
                    && pack.series[0].snapshots.len() == 1
                    && pack.series[0].snapshots[0].row_count == derived.derived_snapshot.row_count;
                record_execution_stage_v2(
                    &mut momentum_trace,
                    JointParticipantExecutionStageV2::EvidencePackConstruction,
                    if pack_valid {
                        JointExecutionStageStatusV2::Completed
                    } else {
                        JointExecutionStageStatusV2::Failed
                    },
                    (!pack_valid).then_some("joint_v2_pack_invariant_invalid"),
                    vec![format!("pack_series_count={}", pack.series.len())],
                    Some(pack.digest.clone()),
                );
                match super::verify_momentum_historical_evidence_pack_v0(&pack) {
                    Err(error) => {
                        record_execution_stage_v2(
                            &mut momentum_trace,
                            JointParticipantExecutionStageV2::EvidencePackVerification,
                            JointExecutionStageStatusV2::Failed,
                            Some("joint_v2_pack_verification_failed"),
                            vec![format!("error={error:?}")],
                            Some(pack.digest.clone()),
                        );
                        momentum_health =
                            JointParticipantExecutionHealthV2::PackVerificationFailure;
                    }
                    Ok(()) if !pack_valid => {
                        record_execution_stage_v2(
                            &mut momentum_trace,
                            JointParticipantExecutionStageV2::EvidencePackVerification,
                            JointExecutionStageStatusV2::Failed,
                            Some("joint_v2_pack_invariant_invalid"),
                            vec![],
                            Some(pack.digest.clone()),
                        );
                        momentum_health =
                            JointParticipantExecutionHealthV2::PackVerificationFailure;
                    }
                    Ok(()) => {
                        record_execution_stage_v2(
                            &mut momentum_trace,
                            JointParticipantExecutionStageV2::EvidencePackVerification,
                            JointExecutionStageStatusV2::Completed,
                            None,
                            vec![],
                            Some(pack.digest.clone()),
                        );
                        match frozen_mamba3_encoder_from_seed_v0(
                            &campaign.feature_config,
                            campaign.campaign_seed,
                            campaign.backend_preference,
                            campaign.fallback_policy,
                        ) {
                            Err(_) => {
                                record_execution_stage_v2(
                                    &mut momentum_trace,
                                    JointParticipantExecutionStageV2::EncoderConstruction,
                                    JointExecutionStageStatusV2::Failed,
                                    Some("joint_v2_encoder_construction_failed"),
                                    vec![],
                                    None,
                                );
                                momentum_health =
                                    JointParticipantExecutionHealthV2::EncoderConstructionFailure;
                            }
                            Ok(encoder) => {
                                record_execution_stage_v2(
                                    &mut momentum_trace,
                                    JointParticipantExecutionStageV2::EncoderConstruction,
                                    JointExecutionStageStatusV2::Completed,
                                    None,
                                    vec![],
                                    Some(encoder.parameter_digest()),
                                );
                                let regime = BtcHistoricalRegimeV0 {
                                    regime_id: scope.joint_scope_id.clone(),
                                    start_row_index: 0,
                                    end_row_index_exclusive: derived.derived_snapshot.row_count,
                                    start_timestamp_ms: derived
                                        .derived_snapshot
                                        .actual_start_timestamp_ms
                                        .ok_or("joint_v2_scope_start_missing")?,
                                    end_timestamp_ms: derived
                                        .derived_snapshot
                                        .actual_end_timestamp_ms
                                        .ok_or("joint_v2_scope_end_missing")?,
                                    row_count: derived.derived_snapshot.row_count,
                                    source_snapshot_id: derived
                                        .derived_snapshot
                                        .snapshot_id
                                        .clone(),
                                    usage_class: EvidenceUsageClassV0::DevelopmentEligible,
                                    segmentation_config_digest: registration
                                        .registration_digest_v2
                                        .clone(),
                                };
                                match run_btc_historical_regime_campaigns_v0(
                                    &[(regime.clone(), pack.clone())],
                                    campaign,
                                    &encoder,
                                ) {
                                    Err(error) => {
                                        record_execution_stage_v2(
                                            &mut momentum_trace,
                                            JointParticipantExecutionStageV2::CampaignExecution,
                                            JointExecutionStageStatusV2::Failed,
                                            Some("joint_v2_campaign_configuration_failed"),
                                            vec![format!("error={error:?}")],
                                            None,
                                        );
                                        momentum_health = JointParticipantExecutionHealthV2::CampaignConfigurationFailure;
                                    }
                                    Ok(results) if results.len() != 1 => {
                                        record_execution_stage_v2(
                                            &mut momentum_trace,
                                            JointParticipantExecutionStageV2::CampaignExecution,
                                            JointExecutionStageStatusV2::Failed,
                                            Some("joint_v2_campaign_output_missing"),
                                            vec![format!("result_count={}", results.len())],
                                            None,
                                        );
                                        momentum_health = JointParticipantExecutionHealthV2::CampaignOutputMissing;
                                    }
                                    Ok(mut results) => {
                                        let result = results.remove(0);
                                        momentum_result_digest = Some(result.report_digest.clone());
                                        momentum_health =
                                            JointParticipantExecutionHealthV2::Completed;
                                        momentum_outcome = momentum_model_outcome_v2(&result);
                                        record_execution_stage_v2(
                                            &mut momentum_trace,
                                            JointParticipantExecutionStageV2::CampaignExecution,
                                            if momentum_outcome == JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal {
                                                JointExecutionStageStatusV2::CompletedNoSignal
                                            } else {
                                                JointExecutionStageStatusV2::Completed
                                            },
                                            None,
                                            vec![format!("campaign_windows={}", result.campaign_windows)],
                                            Some(result.report_digest.clone()),
                                        );
                                        record_execution_stage_v2(
                                            &mut momentum_trace,
                                            JointParticipantExecutionStageV2::ValidationSignalGate,
                                            if momentum_outcome == JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal {
                                                JointExecutionStageStatusV2::CompletedNoSignal
                                            } else {
                                                JointExecutionStageStatusV2::Completed
                                            },
                                            None,
                                            vec![format!("no_signal_windows={}", result.no_signal_windows)],
                                            Some(result.report_digest.clone()),
                                        );
                                        record_execution_stage_v2(
                                            &mut momentum_trace,
                                            JointParticipantExecutionStageV2::CheckpointSelection,
                                            if result.selected_checkpoint_windows == 0 {
                                                JointExecutionStageStatusV2::CompletedNoSignal
                                            } else {
                                                JointExecutionStageStatusV2::Completed
                                            },
                                            None,
                                            vec![format!(
                                                "selected_checkpoint_windows={}",
                                                result.selected_checkpoint_windows
                                            )],
                                            Some(result.report_digest.clone()),
                                        );
                                        record_execution_stage_v2(
                                            &mut momentum_trace,
                                            JointParticipantExecutionStageV2::TemporalDiagnostics,
                                            JointExecutionStageStatusV2::Completed,
                                            None,
                                            vec![],
                                            Some(result.report_digest.clone()),
                                        );
                                        let closed = close_btc_temporal_regime_result_v0(
                                            &result,
                                            BtcTemporalRegimeRefV0 {
                                                regime_id: regime.regime_id,
                                                chronological_rank: scope.chronological_rank,
                                                row_count: regime.row_count,
                                                range_digest: scope.scope_digest_v1.clone(),
                                                pack_digest: pack.digest.clone(),
                                            },
                                        );
                                        match super::validate_btc_temporal_regime_closed_result_v0(
                                            &closed,
                                        ) {
                                            Err(error) => {
                                                let reason_code = match error {
                                                    super::CrossRegimeDiagnosticFailureRootCauseV0::ModelConfigDigestMismatch => "joint_v2_result_closure_model_config_digest_mismatch",
                                                    super::CrossRegimeDiagnosticFailureRootCauseV0::MissingRequiredMetric => "joint_v2_result_closure_missing_required_metric",
                                                    super::CrossRegimeDiagnosticFailureRootCauseV0::PerRegimeReportDigestFailure => "joint_v2_result_closure_report_digest_failure",
                                                    super::CrossRegimeDiagnosticFailureRootCauseV0::CrossRegimeAggregationInvariantFailure => "joint_v2_result_closure_aggregation_invariant_failure",
                                                    super::CrossRegimeDiagnosticFailureRootCauseV0::UnsupportedOutcomeMapping => "joint_v2_result_closure_unsupported_outcome_mapping",
                                                    super::CrossRegimeDiagnosticFailureRootCauseV0::NondeterministicReplay => "joint_v2_result_closure_nondeterministic_replay",
                                                };
                                                record_execution_stage_v2(
                                                    &mut momentum_trace,
                                                    JointParticipantExecutionStageV2::ResultClosure,
                                                    JointExecutionStageStatusV2::Failed,
                                                    Some(reason_code),
                                                    vec![format!("closure_error={error:?}")],
                                                    None,
                                                );
                                                momentum_health = JointParticipantExecutionHealthV2::ResultClosureFailure;
                                                momentum_outcome = JointParticipantModelEvidenceOutcomeV2::NotEvaluatedTechnicalFailure;
                                            }
                                            Ok(()) => {
                                                record_execution_stage_v2(
                                                    &mut momentum_trace,
                                                    JointParticipantExecutionStageV2::ResultClosure,
                                                    JointExecutionStageStatusV2::Completed,
                                                    None,
                                                    vec![],
                                                    Some(closed.report_digest),
                                                );
                                                match momentum_anchor_scope_v1(
                                                    &derived.derived_snapshot,
                                                    0,
                                                    derived.derived_snapshot.row_count,
                                                    campaign,
                                                ) {
                                                    Err(error) => {
                                                        let status = if error.contains("features") {
                                                            JointAnchorAuditStatusV2::FeatureConstructionFailure
                                                        } else if error.contains("examples") {
                                                            JointAnchorAuditStatusV2::SequenceConstructionFailure
                                                        } else if error.contains("windows") {
                                                            JointAnchorAuditStatusV2::WindowConstructionFailure
                                                        } else {
                                                            JointAnchorAuditStatusV2::TechnicalFailure
                                                        };
                                                        momentum_anchor_status = status;
                                                        record_execution_stage_v2(&mut momentum_trace, JointParticipantExecutionStageV2::AnchorMaterialization, JointExecutionStageStatusV2::Failed, Some("joint_v2_momentum_anchor_failed"), vec![error], None);
                                                    }
                                                    Ok(anchors) => {
                                                        momentum_anchor_status = if result
                                                            .selected_checkpoint_windows
                                                            == 0
                                                        {
                                                            JointAnchorAuditStatusV2::CompleteWithoutSelectedCheckpoint
                                                        } else if anchors.effective_anchor_count
                                                            == 0
                                                        {
                                                            JointAnchorAuditStatusV2::NoValidExamples
                                                        } else {
                                                            JointAnchorAuditStatusV2::Complete
                                                        };
                                                        momentum_anchor_digest =
                                                            Some(anchors.scope_digest_v1.clone());
                                                        record_execution_stage_v2(&mut momentum_trace, JointParticipantExecutionStageV2::AnchorMaterialization, if anchors.effective_anchor_count == 0 { JointExecutionStageStatusV2::CompletedAbstained } else { JointExecutionStageStatusV2::Completed }, None, vec![format!("anchor_count={}", anchors.effective_anchor_count)], Some(anchors.scope_digest_v1.clone()));
                                                        if anchors.effective_anchor_count > 0 {
                                                            match make_joint_scope_opinion_v2(campaign.agent_id.clone(), LearnedAgentObjectiveV0::DirectionalMomentum, &derived.derived_snapshot, scope, SourceResultKindV1::MomentumHistoricalRegimeResult, result.report_digest.clone(), stable_hash_string(&format!("momentum-checkpoint:{}", result.report_digest)), pack.digest.clone(), None, encoder.parameter_digest(), &anchors, strings_digest_v1("joint-v2-momentum-forecast", &[campaign.sequence_config.prediction_horizon.to_string(), campaign.sequence_config.label_dead_zone.to_bits().to_string()]), if momentum_outcome == JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal { "completed_no_usable_validation_signal" } else { "completed_retrospective_joint_scope_result" }) {
                                                                Err(error) => record_execution_stage_v2(&mut momentum_trace, JointParticipantExecutionStageV2::SourceBoundOpinionConstruction, JointExecutionStageStatusV2::Failed, Some("joint_v2_momentum_opinion_failed"), vec![error], None),
                                                                Ok(pair) => {
                                                                    record_execution_stage_v2(&mut momentum_trace, JointParticipantExecutionStageV2::SourceBoundOpinionConstruction, JointExecutionStageStatusV2::Completed, None, vec![], Some(pair.0.opinion_digest_v1.clone()));
                                                                    record_execution_stage_v2(&mut momentum_trace, JointParticipantExecutionStageV2::OpinionSeal, JointExecutionStageStatusV2::Completed, None, vec![], Some(pair.1.seal_digest_v1.clone()));
                                                                    momentum_opinion = Some(pair);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    momentum_trace.execution_health = momentum_health;
    momentum_trace.model_evidence_outcome = momentum_outcome;
    momentum_trace.operational_shadow_result =
        operational_shadow_result_v2(momentum_health, momentum_outcome);
    finish_execution_trace_v2(&mut momentum_trace);
    let mut momentum = JointScopeParticipantReplayResultV2 {
        joint_scope_id: scope.joint_scope_id.clone(),
        joint_scope_digest: scope.scope_digest_v1.clone(),
        participant_agent_id: campaign.agent_id.clone(),
        objective: LearnedAgentObjectiveV0::DirectionalMomentum,
        execution_trace: momentum_trace,
        completed_result_digest: momentum_result_digest,
        anchor_scope_digest: momentum_anchor_digest,
        anchor_status: momentum_anchor_status,
        opinion_id: momentum_opinion
            .as_ref()
            .map(|pair| pair.0.opinion_id.clone()),
        seal_digest: momentum_opinion
            .as_ref()
            .map(|pair| pair.1.seal_digest_v1.clone()),
        sealed_opinion: momentum_opinion,
        result_digest_v2: String::new(),
    };
    momentum.result_digest_v2 = participant_result_digest_v2(&momentum);

    let mut risk_trace = new_execution_trace_v2(
        scope,
        CYCLE_RISK_SHADOW_AGENT_ID_V0.into(),
        LearnedAgentObjectiveV0::DownsideRisk,
    );
    for (stage, digest) in [
        (
            JointParticipantExecutionStageV2::ParentSnapshotVerification,
            snapshot.content_digest.clone(),
        ),
        (
            JointParticipantExecutionStageV2::JointScopeVerification,
            scope.scope_digest_v1.clone(),
        ),
        (
            JointParticipantExecutionStageV2::DerivedSnapshotConstruction,
            derived.derivation_digest_v2.clone(),
        ),
        (
            JointParticipantExecutionStageV2::DerivedSnapshotIdentity,
            derived.derived_snapshot.snapshot_id.clone(),
        ),
        (
            JointParticipantExecutionStageV2::DerivedSnapshotSemanticVerification,
            derived.derivation_proof.proof_digest_v2.clone(),
        ),
    ] {
        record_execution_stage_v2(
            &mut risk_trace,
            stage,
            JointExecutionStageStatusV2::Completed,
            None,
            vec![],
            Some(digest),
        );
    }
    let risk_config = CycleRiskShadowConfigV0::default();
    let risk_health;
    let risk_outcome;
    let mut risk_result_digest = None;
    let mut risk_anchor_digest = None;
    let mut risk_anchor_status = JointAnchorAuditStatusV2::TechnicalFailure;
    let mut risk_opinion = None;
    match run_cycle_risk_shadow_regime_v0(
        &derived.derived_snapshot,
        &scope.joint_scope_id,
        &risk_config,
    ) {
        Err(error) => {
            record_execution_stage_v2(
                &mut risk_trace,
                JointParticipantExecutionStageV2::CampaignExecution,
                JointExecutionStageStatusV2::Failed,
                Some("joint_v2_risk_campaign_failed"),
                vec![format!("error={error:?}")],
                None,
            );
            risk_health = JointParticipantExecutionHealthV2::CampaignRuntimeFailure;
            risk_outcome = JointParticipantModelEvidenceOutcomeV2::NotEvaluatedTechnicalFailure;
        }
        Ok(result) => {
            let digest = stable_hash_string(&format!("{:?}", result));
            risk_result_digest = Some(digest.clone());
            risk_health = JointParticipantExecutionHealthV2::Completed;
            risk_outcome = match result.verdict {
                super::CycleRiskShadowVerdictV0::PositiveEvidence => {
                    JointParticipantModelEvidenceOutcomeV2::UsableValidationSignal
                }
                super::CycleRiskShadowVerdictV0::InsufficientEvents
                | super::CycleRiskShadowVerdictV0::ShadowOnly => {
                    JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal
                }
                super::CycleRiskShadowVerdictV0::ProbabilityCollapse => {
                    JointParticipantModelEvidenceOutcomeV2::ProbabilityCollapse
                }
                super::CycleRiskShadowVerdictV0::LinearBaselineStronger
                | super::CycleRiskShadowVerdictV0::ConstantBaselineStronger => {
                    JointParticipantModelEvidenceOutcomeV2::BaselineStronger
                }
                super::CycleRiskShadowVerdictV0::HighConfidenceFalseNegative => {
                    JointParticipantModelEvidenceOutcomeV2::ValidationSignalOutOfSupport
                }
            };
            record_execution_stage_v2(
                &mut risk_trace,
                JointParticipantExecutionStageV2::CampaignExecution,
                if risk_outcome == JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal
                {
                    JointExecutionStageStatusV2::CompletedNoSignal
                } else {
                    JointExecutionStageStatusV2::Completed
                },
                None,
                vec![],
                Some(digest.clone()),
            );
            let candidate = CycleRiskHistoricalRangeCandidateV0 {
                candidate_range_id: scope.joint_scope_id.clone(),
                start_row_index: 0,
                end_row_index_exclusive: derived.derived_snapshot.row_count,
                row_count: derived.derived_snapshot.row_count,
                canonical_scope_digest_v1: scope.canonical_raw_scope.scope_digest_v1.clone(),
                expected_frozen_pack_digest: result.frozen_pack_digest.clone(),
                range_digest: scope.scope_digest_v1.clone(),
            };
            match risk_anchor_scope_v1(&derived.derived_snapshot, &candidate, &risk_config) {
                Err(error) => {
                    risk_anchor_status = JointAnchorAuditStatusV2::TechnicalFailure;
                    record_execution_stage_v2(
                        &mut risk_trace,
                        JointParticipantExecutionStageV2::AnchorMaterialization,
                        JointExecutionStageStatusV2::Failed,
                        Some("joint_v2_risk_anchor_failed"),
                        vec![error],
                        None,
                    );
                }
                Ok(anchors) => {
                    risk_anchor_status = if anchors.effective_anchor_count == 0 {
                        JointAnchorAuditStatusV2::NoValidExamples
                    } else {
                        JointAnchorAuditStatusV2::Complete
                    };
                    risk_anchor_digest = Some(anchors.scope_digest_v1.clone());
                    record_execution_stage_v2(
                        &mut risk_trace,
                        JointParticipantExecutionStageV2::AnchorMaterialization,
                        if anchors.effective_anchor_count == 0 {
                            JointExecutionStageStatusV2::CompletedAbstained
                        } else {
                            JointExecutionStageStatusV2::Completed
                        },
                        None,
                        vec![format!("anchor_count={}", anchors.effective_anchor_count)],
                        Some(anchors.scope_digest_v1.clone()),
                    );
                    if anchors.effective_anchor_count > 0 {
                        match make_joint_scope_opinion_v2(
                            CYCLE_RISK_SHADOW_AGENT_ID_V0.into(),
                            LearnedAgentObjectiveV0::DownsideRisk,
                            &derived.derived_snapshot,
                            scope,
                            SourceResultKindV1::CycleRiskHistoricalRegimeResult,
                            digest,
                            stable_hash_string(&format!(
                                "risk-checkpoint:{}",
                                result.frozen_pack_digest
                            )),
                            result.frozen_pack_digest.clone(),
                            result.checkpoint.accepted_model_version.clone(),
                            result
                                .checkpoint
                                .accepted_model_version
                                .clone()
                                .unwrap_or_else(|| result.frozen_pack_digest.clone()),
                            &anchors,
                            strings_digest_v1(
                                "joint-v2-risk-forecast",
                                &[
                                    risk_config.label.horizon_rows.to_string(),
                                    risk_config.label.digest(),
                                ],
                            ),
                            if risk_outcome
                                == JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal
                            {
                                "completed_no_usable_validation_signal"
                            } else {
                                "completed_retrospective_joint_scope_result"
                            },
                        ) {
                            Err(error) => record_execution_stage_v2(
                                &mut risk_trace,
                                JointParticipantExecutionStageV2::SourceBoundOpinionConstruction,
                                JointExecutionStageStatusV2::Failed,
                                Some("joint_v2_risk_opinion_failed"),
                                vec![error],
                                None,
                            ),
                            Ok(pair) => {
                                record_execution_stage_v2(&mut risk_trace, JointParticipantExecutionStageV2::SourceBoundOpinionConstruction, JointExecutionStageStatusV2::Completed, None, vec![], Some(pair.0.opinion_digest_v1.clone()));
                                record_execution_stage_v2(
                                    &mut risk_trace,
                                    JointParticipantExecutionStageV2::OpinionSeal,
                                    JointExecutionStageStatusV2::Completed,
                                    None,
                                    vec![],
                                    Some(pair.1.seal_digest_v1.clone()),
                                );
                                risk_opinion = Some(pair);
                            }
                        }
                    }
                }
            }
        }
    }
    risk_trace.execution_health = risk_health;
    risk_trace.model_evidence_outcome = risk_outcome;
    risk_trace.operational_shadow_result = operational_shadow_result_v2(risk_health, risk_outcome);
    finish_execution_trace_v2(&mut risk_trace);
    let mut risk = JointScopeParticipantReplayResultV2 {
        joint_scope_id: scope.joint_scope_id.clone(),
        joint_scope_digest: scope.scope_digest_v1.clone(),
        participant_agent_id: CYCLE_RISK_SHADOW_AGENT_ID_V0.into(),
        objective: LearnedAgentObjectiveV0::DownsideRisk,
        execution_trace: risk_trace,
        completed_result_digest: risk_result_digest,
        anchor_scope_digest: risk_anchor_digest,
        anchor_status: risk_anchor_status,
        opinion_id: risk_opinion.as_ref().map(|pair| pair.0.opinion_id.clone()),
        seal_digest: risk_opinion
            .as_ref()
            .map(|pair| pair.1.seal_digest_v1.clone()),
        sealed_opinion: risk_opinion,
        result_digest_v2: String::new(),
    };
    risk.result_digest_v2 = participant_result_digest_v2(&risk);
    let pair_eligible = momentum.execution_trace.execution_health
        == JointParticipantExecutionHealthV2::Completed
        && risk.execution_trace.execution_health == JointParticipantExecutionHealthV2::Completed
        && momentum.sealed_opinion.is_some()
        && risk.sealed_opinion.is_some()
        && momentum.sealed_opinion.as_ref().is_some_and(|pair| {
            pair.0.source_result.canonical_raw_row_identity_digests_v1
                == scope.canonical_raw_scope.row_identity_digests
        })
        && risk.sealed_opinion.as_ref().is_some_and(|pair| {
            pair.0.source_result.canonical_raw_row_identity_digests_v1
                == scope.canonical_raw_scope.row_identity_digests
        });
    let result_digest_v2 = joint_v2_digest(&[
        registration.registration_digest_v2.clone(),
        scope.joint_scope_id.clone(),
        scope.scope_digest_v1.clone(),
        derived.derived_snapshot.snapshot_id.clone(),
        evidence_policy.policy_digest_v2.clone(),
        momentum.result_digest_v2.clone(),
        risk.result_digest_v2.clone(),
        pair_eligible.to_string(),
    ]);
    Ok(JointScopeReplayResultV2 {
        replay_version: "joint-canonical-scope-replay-v2".into(),
        registration_digest_v2: registration.registration_digest_v2.clone(),
        joint_scope_id: scope.joint_scope_id.clone(),
        joint_scope_digest: scope.scope_digest_v1.clone(),
        derived_snapshot_id: derived.derived_snapshot.snapshot_id,
        derivation_digest_v2: derived.derivation_digest_v2,
        evidence_policy_digest_v2: evidence_policy.policy_digest_v2,
        momentum,
        risk,
        pair_eligible,
        result_digest_v2,
    })
}

pub fn aggregate_joint_scope_replays_v2(
    registration: &JointCanonicalScopeReplayRegistrationV2,
    results: &[JointScopeReplayResultV2],
) -> Result<(JointScopeReplayAggregateV2, JointScopeReplayLedgerV2), String> {
    if registration.registration_digest_v2 != joint_v2_registration_digest(registration)
        || results.iter().any(|result| {
            result.registration_digest_v2 != registration.registration_digest_v2
                || !registration
                    .joint_scope_ids
                    .contains(&result.joint_scope_id)
                || !registration
                    .joint_scope_digests
                    .contains(&result.joint_scope_digest)
        })
    {
        return Err("joint_v2_aggregate_registration_mismatch".into());
    }
    let mut ordered = results.to_vec();
    ordered.sort_by(|left, right| left.joint_scope_id.cmp(&right.joint_scope_id));
    let mut both_abstained_count = 0usize;
    let mut momentum_abstained_count = 0usize;
    let mut risk_abstained_count = 0usize;
    let mut technical_failure_scope_count = 0usize;
    let mut incomparable_count = 0usize;
    let mut tension_count = 0usize;
    let mut orthogonal_count = 0usize;
    let mut deliberations = Vec::new();
    for result in &ordered {
        if result.momentum.execution_trace.execution_health
            != JointParticipantExecutionHealthV2::Completed
            || result.risk.execution_trace.execution_health
                != JointParticipantExecutionHealthV2::Completed
        {
            technical_failure_scope_count += 1;
            continue;
        }
        match (
            result.momentum.execution_trace.operational_shadow_result,
            result.risk.execution_trace.operational_shadow_result,
        ) {
            (
                JointParticipantOperationalShadowResultV2::ShadowAbstainNoSignal,
                JointParticipantOperationalShadowResultV2::ShadowAbstainNoSignal,
            ) => both_abstained_count += 1,
            (JointParticipantOperationalShadowResultV2::ShadowAbstainNoSignal, _) => {
                momentum_abstained_count += 1
            }
            (_, JointParticipantOperationalShadowResultV2::ShadowAbstainNoSignal) => {
                risk_abstained_count += 1
            }
            _ => {}
        }
        let (Some(momentum), Some(risk)) = (
            result.momentum.sealed_opinion.as_ref(),
            result.risk.sealed_opinion.as_ref(),
        ) else {
            incomparable_count += 1;
            continue;
        };
        let mapping = map_source_bound_opinions_v1(
            std::slice::from_ref(momentum),
            std::slice::from_ref(risk),
        )?;
        let Some(pair) = mapping.scope_pairs.first() else {
            incomparable_count += 1;
            continue;
        };
        let momentum_abstains = !matches!(
            result.momentum.execution_trace.operational_shadow_result,
            JointParticipantOperationalShadowResultV2::ShadowPredictionResearchOnly
        );
        let risk_abstains = !matches!(
            result.risk.execution_trace.operational_shadow_result,
            JointParticipantOperationalShadowResultV2::ShadowPredictionResearchOnly
        );
        let relationship = match (momentum_abstains, risk_abstains) {
            (true, true) => JointScopeRelationshipV2::BothAbstained,
            (true, false) => JointScopeRelationshipV2::MomentumAbstained,
            (false, true) => JointScopeRelationshipV2::RiskAbstained,
            (false, false) => JointScopeRelationshipV2::Orthogonal,
        };
        if relationship == JointScopeRelationshipV2::Orthogonal {
            orthogonal_count += 1;
        }
        let transcript_digest_v2 = joint_v2_digest(&[
            result.joint_scope_id.clone(),
            pair.pair_digest_v1.clone(),
            format!("{relationship:?}"),
            "round_count=2".into(),
            "retrospective_only=true".into(),
            "authority=false".into(),
        ]);
        deliberations.push(JointScopeDeliberationV2 {
            deliberation_version: "joint-scope-deliberation-v2".into(),
            joint_scope_id: result.joint_scope_id.clone(),
            mapping_pair_digest_v1: pair.pair_digest_v1.clone(),
            relationship,
            round_count: 2,
            retrospective_only: true,
            chair_observed: false,
            vote_created: false,
            reward_created: false,
            penalty_created: false,
            execution_created: false,
            transcript_digest_v2,
        });
    }
    deliberations.sort_by(|left, right| left.joint_scope_id.cmp(&right.joint_scope_id));
    let completed_pair_count = deliberations.len();
    let transcript_digests = deliberations
        .iter()
        .map(|value| value.transcript_digest_v2.clone())
        .collect::<Vec<_>>();
    let relationships = deliberations
        .iter()
        .map(|value| value.relationship)
        .collect::<Vec<_>>();
    let full_aggregate_composed = ordered.len() == registration.joint_scope_ids.len()
        && completed_pair_count == registration.joint_scope_ids.len()
        && technical_failure_scope_count == 0;
    let mut aggregate = JointScopeReplayAggregateV2 {
        aggregate_version: "joint-scope-relationship-only-aggregate-v2".into(),
        registration_digest_v2: registration.registration_digest_v2.clone(),
        completed_pair_count,
        technical_failure_scope_count,
        both_abstained_count,
        momentum_abstained_count,
        risk_abstained_count,
        tension_count,
        orthogonal_count,
        incomparable_count,
        relationship_count: completed_pair_count,
        deliberation_count: completed_pair_count,
        transcript_digests: transcript_digests.clone(),
        relationships: relationships.clone(),
        deliberations: deliberations.clone(),
        full_aggregate_composed,
        aggregate_digest_v2: String::new(),
    };
    aggregate.aggregate_digest_v2 = joint_v2_digest(&[
        aggregate.registration_digest_v2.clone(),
        aggregate.completed_pair_count.to_string(),
        aggregate.technical_failure_scope_count.to_string(),
        aggregate.both_abstained_count.to_string(),
        aggregate.momentum_abstained_count.to_string(),
        aggregate.risk_abstained_count.to_string(),
        aggregate.tension_count.to_string(),
        aggregate.orthogonal_count.to_string(),
        aggregate.incomparable_count.to_string(),
        aggregate.relationship_count.to_string(),
        aggregate.deliberation_count.to_string(),
        aggregate.transcript_digests.join(":"),
        aggregate
            .relationships
            .iter()
            .map(|value| format!("{value:?}"))
            .collect::<Vec<_>>()
            .join(":"),
        aggregate.full_aggregate_composed.to_string(),
    ]);
    let mut participant_result_digests = ordered
        .iter()
        .flat_map(|result| {
            [
                result.momentum.result_digest_v2.clone(),
                result.risk.result_digest_v2.clone(),
            ]
        })
        .collect::<Vec<_>>();
    participant_result_digests.sort();
    let mut ledger = JointScopeReplayLedgerV2 {
        ledger_version: "joint-scope-replay-ledger-v2".into(),
        registration_digest_v2: registration.registration_digest_v2.clone(),
        participant_result_digests,
        deliberation_transcript_digests: transcript_digests,
        ledger_digest_v2: String::new(),
    };
    ledger.ledger_digest_v2 = joint_v2_digest(&[
        ledger.ledger_version.clone(),
        ledger.registration_digest_v2.clone(),
        ledger.participant_result_digests.join(":"),
        ledger.deliberation_transcript_digests.join(":"),
    ]);
    Ok((aggregate, ledger))
}

pub fn validate_joint_scope_replay_ledger_v2(
    ledger: &JointScopeReplayLedgerV2,
) -> Result<(), String> {
    if ledger.ledger_version != "joint-scope-replay-ledger-v2"
        || ledger
            .participant_result_digests
            .windows(2)
            .any(|pair| pair[0] > pair[1])
        || ledger.ledger_digest_v2
            != joint_v2_digest(&[
                ledger.ledger_version.clone(),
                ledger.registration_digest_v2.clone(),
                ledger.participant_result_digests.join(":"),
                ledger.deliberation_transcript_digests.join(":"),
            ])
    {
        return Err("joint_v2_replay_ledger_invalid".into());
    }
    Ok(())
}

pub fn interpret_sprint57_momentum_outcome_v2(
    scope: &JointCanonicalHistoricalScopeV1,
    forensic: &JointMomentumForensicReportV2,
    corrected: &JointScopeParticipantReplayResultV2,
) -> Result<Sprint57MomentumOutcomeInterpretationV2, String> {
    if scope.joint_scope_id != forensic.joint_scope_id
        || scope.joint_scope_id != corrected.joint_scope_id
    {
        return Err("sprint57_interpretation_scope_mismatch".into());
    }
    let mut value = Sprint57MomentumOutcomeInterpretationV2 {
        sprint57_registration_digest: scope.registration_digest_v1.clone(),
        joint_scope_id: scope.joint_scope_id.clone(),
        legacy_reported_status: "LegacyCollapsedOutcome".into(),
        forensic_root_cause: forensic.root_cause,
        corrected_execution_health: corrected.execution_trace.execution_health,
        corrected_model_outcome: corrected.execution_trace.model_evidence_outcome,
        sprint57_artifact_mutated: false,
        interpretation_digest_v2: String::new(),
    };
    value.interpretation_digest_v2 = joint_v2_digest(&[
        value.sprint57_registration_digest.clone(),
        value.joint_scope_id.clone(),
        value.legacy_reported_status.clone(),
        format!("{:?}", value.forensic_root_cause),
        format!("{:?}", value.corrected_execution_health),
        format!("{:?}", value.corrected_model_outcome),
        value.sprint57_artifact_mutated.to_string(),
    ]);
    Ok(value)
}

pub fn source_bound_shadow_ledger_record_v1(
    ledger: &SourceBoundShadowDeliberationLedgerV1,
) -> Result<SourceBoundShadowLedgerRecordV1, String> {
    if ledger.opinions.len() != ledger.opinion_seals.len()
        || ledger
            .opinions
            .iter()
            .zip(&ledger.opinion_seals)
            .any(|(opinion, seal)| opinion.opinion_id != seal.opinion_id)
    {
        return Err("source_bound_ledger_record_invalid".into());
    }
    let record = SourceBoundShadowLedgerRecordV1 {
        ledger_version: ledger.ledger_version.clone(),
        protocol_registration_digest_v1: ledger.protocol_registration_digest_v1.clone(),
        scope_mapping_registry_digest_v1: ledger.scope_mapping_registry_digest_v1.clone(),
        legacy_v0_reference_digest: ledger.legacy_v0_reference_digest.clone(),
        opinions: ledger
            .opinions
            .iter()
            .zip(&ledger.opinion_seals)
            .map(|(opinion, seal)| SourceBoundLedgerOpinionRecordV1 {
                opinion_id: opinion.opinion_id.clone(),
                opinion_digest_v1: opinion.opinion_digest_v1.clone(),
                seal_digest_v1: seal.seal_digest_v1.clone(),
            })
            .collect(),
        ledger_digest_v1: ledger.ledger_digest_v1.clone(),
    };
    validate_source_bound_shadow_ledger_record_v1(&record)?;
    Ok(record)
}

pub fn validate_source_bound_shadow_ledger_record_v1(
    record: &SourceBoundShadowLedgerRecordV1,
) -> Result<(), String> {
    if record.ledger_version != "source-bound-shadow-ledger-v1"
        || record.protocol_registration_digest_v1
            != SourceBoundOpinionProtocolRegistrationV1::pre_registered().policy_digest_v1
        || record
            .opinions
            .windows(2)
            .any(|pair| pair[0].opinion_id >= pair[1].opinion_id)
        || record.opinions.iter().any(|entry| {
            entry.opinion_id.is_empty()
                || entry.opinion_digest_v1.is_empty()
                || entry.seal_digest_v1.is_empty()
        })
    {
        return Err("source_bound_ledger_record_invalid".into());
    }
    let mut bytes = Vec::new();
    strv(&mut bytes, &record.ledger_version);
    strv(&mut bytes, &record.protocol_registration_digest_v1);
    strv(&mut bytes, &record.scope_mapping_registry_digest_v1);
    strv(&mut bytes, &record.legacy_v0_reference_digest);
    strings(
        &mut bytes,
        &record
            .opinions
            .iter()
            .map(|entry| entry.opinion_digest_v1.clone())
            .collect::<Vec<_>>(),
    );
    strings(
        &mut bytes,
        &record
            .opinions
            .iter()
            .map(|entry| entry.seal_digest_v1.clone())
            .collect::<Vec<_>>(),
    );
    (record.ledger_digest_v1 == stable_hash_string(&hex(&bytes)))
        .then_some(())
        .ok_or_else(|| "source_bound_ledger_record_digest_invalid".into())
}

pub fn write_source_bound_shadow_ledger_record_v1(
    path: &Path,
    ledger: &SourceBoundShadowDeliberationLedgerV1,
) -> Result<(), String> {
    let record = source_bound_shadow_ledger_record_v1(ledger)?;
    let parent = path
        .parent()
        .ok_or_else(|| "source_bound_ledger_record_path_invalid".to_string())?;
    fs::create_dir_all(parent).map_err(|_| "source_bound_ledger_record_storage".to_string())?;
    let encoded = serde_json::to_vec(&record)
        .map_err(|_| "source_bound_ledger_record_storage".to_string())?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, encoded).map_err(|_| "source_bound_ledger_record_storage".to_string())?;
    fs::rename(&temporary, path).map_err(|_| "source_bound_ledger_record_storage".to_string())
}

pub fn read_source_bound_shadow_ledger_record_v1(
    path: &Path,
) -> Result<SourceBoundShadowLedgerRecordV1, String> {
    let encoded = fs::read(path).map_err(|_| "source_bound_ledger_record_storage".to_string())?;
    let record = serde_json::from_slice(&encoded)
        .map_err(|_| "source_bound_ledger_record_storage".to_string())?;
    validate_source_bound_shadow_ledger_record_v1(&record)?;
    Ok(record)
}

/// A deliberately narrow, retrospective-only representation of the V3 replay.
///
/// This module intentionally does not depend on Chair runtime types. The packet is
/// therefore not adaptable to a vote, Chair input, signal, proposal, or output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairObservationEvidenceClassV0 {
    RetrospectiveDevelopmentOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairShadowObservationAuthorityV0 {
    pub advisory_only: bool,
    pub observation_only: bool,
    pub chair_decision_allowed: bool,
    pub vote_allowed: bool,
    pub speaker_selection_allowed: bool,
    pub reward_or_penalty_allowed: bool,
    pub speaking_right_change_allowed: bool,
    pub risk_handoff_allowed: bool,
    pub execution_allowed: bool,
}

impl ChairShadowObservationAuthorityV0 {
    pub fn retrospective_observation_only() -> Self {
        Self {
            advisory_only: true,
            observation_only: true,
            chair_decision_allowed: false,
            vote_allowed: false,
            speaker_selection_allowed: false,
            reward_or_penalty_allowed: false,
            speaking_right_change_allowed: false,
            risk_handoff_allowed: false,
            execution_allowed: false,
        }
    }

    fn is_exact_retrospective_observation_only(&self) -> bool {
        self == &Self::retrospective_observation_only()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairShadowObservationPacketV0 {
    pub packet_version: String,
    pub source_replay_version: String,
    pub source_registration_digest: String,
    pub source_ledger_digest: String,
    pub source_aggregate_digest: String,
    pub opinion_ids: Vec<String>,
    pub opinion_seal_digests: Vec<String>,
    pub relationship_digests: Vec<String>,
    pub transcript_digests: Vec<String>,
    pub evidence_class: ChairObservationEvidenceClassV0,
    pub retrospective_only: bool,
    pub prospective: bool,
    pub authority: ChairShadowObservationAuthorityV0,
    pub packet_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairObservationReceiptStatusV0 {
    AcceptedRetrospectiveObservationOnly,
    InvalidRegistration,
    InvalidLedger,
    InvalidAggregate,
    InvalidOpinionSeal,
    InvalidTranscript,
    AuthorityViolation,
    ProspectiveClaimForbidden,
    DuplicatePacket,
    TechnicalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairObservedRelationshipCategoryV0 {
    BothAbstained,
    MomentumAbstained,
    RiskAbstained,
    Tension,
    Orthogonal,
    Incomparable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairObservedRelationshipCountV0 {
    pub category: ChairObservedRelationshipCategoryV0,
    pub count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChairObservationUncertaintyCategoryV0 {
    RetrospectiveDevelopmentOnly,
    AbstentionObserved,
    NoDecisionAuthority,
    NoExecutionAuthority,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairObservationUncertaintyV0 {
    pub category: ChairObservationUncertaintyCategoryV0,
    pub present: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairShadowObservationReceiptV0 {
    pub receipt_version: String,
    pub packet_digest: String,
    pub status: ChairObservationReceiptStatusV0,
    pub observed_agent_ids: Vec<String>,
    pub observed_objectives: Vec<String>,
    pub observed_scope_count: usize,
    pub observed_opinion_count: usize,
    pub observed_abstention_count: usize,
    pub relationship_summary: Vec<ChairObservedRelationshipCountV0>,
    pub scope_caveats: Vec<String>,
    pub uncertainty_flags: Vec<ChairObservationUncertaintyV0>,
    pub source_aggregate_digest: String,
    pub source_ledger_digest: String,
    pub chair_runtime_invocations: usize,
    pub chair_decisions_created: usize,
    pub votes_created: usize,
    pub rewards_created: usize,
    pub penalties_created: usize,
    pub speaking_right_changes: usize,
    pub risk_handoffs: usize,
    pub executions_created: usize,
    pub receipt_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairShadowObservationInboxV0 {
    pub inbox_version: String,
    pub packets: Vec<ChairShadowObservationPacketV0>,
    pub accepted_packet_ids: Vec<String>,
    pub rejected_packet_ids: Vec<String>,
    pub chair_runtime_invocations: usize,
    pub chair_decisions_created: usize,
    pub votes_created: usize,
    pub rewards_created: usize,
    pub penalties_created: usize,
    pub speaking_right_changes: usize,
    pub risk_handoffs: usize,
    pub executions_created: usize,
    pub inbox_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairShadowDecisionFirewallProofV0 {
    pub packet_cannot_become_vote: bool,
    pub packet_cannot_become_chair_input: bool,
    pub chair_engine_not_invoked: bool,
    pub speaker_selection_not_invoked: bool,
    pub council_score_not_computed: bool,
    pub decision_not_created: bool,
    pub size_multiplier_not_created: bool,
    pub risk_handoff_not_created: bool,
    pub execution_not_created: bool,
    pub all_invariants_pass: bool,
    pub proof_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairShadowObservationStorageV0 {
    pub storage_version: String,
    pub inbox: ChairShadowObservationInboxV0,
    pub receipts: Vec<ChairShadowObservationReceiptV0>,
    pub firewall_proofs: Vec<ChairShadowDecisionFirewallProofV0>,
    pub storage_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChairShadowObservationReportV0 {
    pub report_version: String,
    pub offline: bool,
    pub active_committee_count: usize,
    pub packet: ChairShadowObservationPacketV0,
    pub inbox: ChairShadowObservationInboxV0,
    pub receipt: ChairShadowObservationReceiptV0,
    pub firewall_proof: ChairShadowDecisionFirewallProofV0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChairShadowObservationEvidenceV0 {
    registration: JointCanonicalScopeReplayRegistrationV3,
    results: Vec<JointScopeReplayResultV3>,
    aggregate: JointScopeReplayAggregateV3,
    ledger: JointScopeReplayLedgerV3,
}

fn chair_relationship_category_v0(
    relationship: JointScopeRelationshipV2,
) -> ChairObservedRelationshipCategoryV0 {
    match relationship {
        JointScopeRelationshipV2::BothAbstained => {
            ChairObservedRelationshipCategoryV0::BothAbstained
        }
        JointScopeRelationshipV2::MomentumAbstained => {
            ChairObservedRelationshipCategoryV0::MomentumAbstained
        }
        JointScopeRelationshipV2::RiskAbstained => {
            ChairObservedRelationshipCategoryV0::RiskAbstained
        }
        JointScopeRelationshipV2::Tension => ChairObservedRelationshipCategoryV0::Tension,
        JointScopeRelationshipV2::Orthogonal => ChairObservedRelationshipCategoryV0::Orthogonal,
        JointScopeRelationshipV2::Incomparable => ChairObservedRelationshipCategoryV0::Incomparable,
    }
}

fn chair_relationship_digest_v0(value: &JointScopeDeliberationV2) -> String {
    joint_v3_digest(&[
        "chair-shadow-observed-relationship-v0".into(),
        value.joint_scope_id.clone(),
        value.mapping_pair_digest_v1.clone(),
        format!("{:?}", value.relationship),
        value.transcript_digest_v2.clone(),
    ])
}

fn chair_packet_digest_v0(value: &ChairShadowObservationPacketV0) -> String {
    joint_v3_digest(&[
        value.packet_version.clone(),
        value.source_replay_version.clone(),
        value.source_registration_digest.clone(),
        value.source_ledger_digest.clone(),
        value.source_aggregate_digest.clone(),
        value.opinion_ids.join(":"),
        value.opinion_seal_digests.join(":"),
        value.relationship_digests.join(":"),
        value.transcript_digests.join(":"),
        format!("{:?}", value.evidence_class),
        value.retrospective_only.to_string(),
        value.prospective.to_string(),
        value.authority.advisory_only.to_string(),
        value.authority.observation_only.to_string(),
        value.authority.chair_decision_allowed.to_string(),
        value.authority.vote_allowed.to_string(),
        value.authority.speaker_selection_allowed.to_string(),
        value.authority.reward_or_penalty_allowed.to_string(),
        value.authority.speaking_right_change_allowed.to_string(),
        value.authority.risk_handoff_allowed.to_string(),
        value.authority.execution_allowed.to_string(),
    ])
}

fn chair_inbox_digest_v0(value: &ChairShadowObservationInboxV0) -> String {
    joint_v3_digest(&[
        value.inbox_version.clone(),
        value
            .packets
            .iter()
            .map(|packet| packet.packet_digest.clone())
            .collect::<Vec<_>>()
            .join(":"),
        value.accepted_packet_ids.join(":"),
        value.rejected_packet_ids.join(":"),
        value.chair_runtime_invocations.to_string(),
        value.chair_decisions_created.to_string(),
        value.votes_created.to_string(),
        value.rewards_created.to_string(),
        value.penalties_created.to_string(),
        value.speaking_right_changes.to_string(),
        value.risk_handoffs.to_string(),
        value.executions_created.to_string(),
    ])
}

fn chair_receipt_digest_v0(value: &ChairShadowObservationReceiptV0) -> String {
    joint_v3_digest(&[
        value.receipt_version.clone(),
        value.packet_digest.clone(),
        format!("{:?}", value.status),
        value.observed_agent_ids.join(":"),
        value.observed_objectives.join(":"),
        value.observed_scope_count.to_string(),
        value.observed_opinion_count.to_string(),
        value.observed_abstention_count.to_string(),
        value
            .relationship_summary
            .iter()
            .map(|item| format!("{:?}:{}", item.category, item.count))
            .collect::<Vec<_>>()
            .join(":"),
        value.scope_caveats.join(":"),
        value
            .uncertainty_flags
            .iter()
            .map(|item| format!("{:?}:{}", item.category, item.present))
            .collect::<Vec<_>>()
            .join(":"),
        value.source_aggregate_digest.clone(),
        value.source_ledger_digest.clone(),
        value.chair_runtime_invocations.to_string(),
        value.chair_decisions_created.to_string(),
        value.votes_created.to_string(),
        value.rewards_created.to_string(),
        value.penalties_created.to_string(),
        value.speaking_right_changes.to_string(),
        value.risk_handoffs.to_string(),
        value.executions_created.to_string(),
    ])
}

fn chair_firewall_proof_v0() -> ChairShadowDecisionFirewallProofV0 {
    let mut proof = ChairShadowDecisionFirewallProofV0 {
        packet_cannot_become_vote: true,
        packet_cannot_become_chair_input: true,
        chair_engine_not_invoked: true,
        speaker_selection_not_invoked: true,
        council_score_not_computed: true,
        decision_not_created: true,
        size_multiplier_not_created: true,
        risk_handoff_not_created: true,
        execution_not_created: true,
        all_invariants_pass: true,
        proof_digest: String::new(),
    };
    proof.proof_digest = joint_v3_digest(&[
        "chair-shadow-decision-firewall-proof-v0".into(),
        proof.packet_cannot_become_vote.to_string(),
        proof.packet_cannot_become_chair_input.to_string(),
        proof.chair_engine_not_invoked.to_string(),
        proof.speaker_selection_not_invoked.to_string(),
        proof.council_score_not_computed.to_string(),
        proof.decision_not_created.to_string(),
        proof.size_multiplier_not_created.to_string(),
        proof.risk_handoff_not_created.to_string(),
        proof.execution_not_created.to_string(),
        proof.all_invariants_pass.to_string(),
    ]);
    proof
}

fn chair_storage_digest_v0(value: &ChairShadowObservationStorageV0) -> String {
    joint_v3_digest(&[
        value.storage_version.clone(),
        value.inbox.inbox_digest.clone(),
        value
            .receipts
            .iter()
            .map(|receipt| receipt.receipt_digest.clone())
            .collect::<Vec<_>>()
            .join(":"),
        value
            .firewall_proofs
            .iter()
            .map(|proof| proof.proof_digest.clone())
            .collect::<Vec<_>>()
            .join(":"),
    ])
}

pub fn new_chair_shadow_observation_inbox_v0() -> ChairShadowObservationInboxV0 {
    let mut inbox = ChairShadowObservationInboxV0 {
        inbox_version: "chair-shadow-observation-inbox-v0".into(),
        packets: vec![],
        accepted_packet_ids: vec![],
        rejected_packet_ids: vec![],
        chair_runtime_invocations: 0,
        chair_decisions_created: 0,
        votes_created: 0,
        rewards_created: 0,
        penalties_created: 0,
        speaking_right_changes: 0,
        risk_handoffs: 0,
        executions_created: 0,
        inbox_digest: String::new(),
    };
    inbox.inbox_digest = chair_inbox_digest_v0(&inbox);
    inbox
}

fn chair_source_bindings_v0(
    evidence: &ChairShadowObservationEvidenceV0,
) -> Result<Vec<(String, String, String, String, bool)>, String> {
    let registration = SourceBoundOpinionProtocolRegistrationV1::pre_registered();
    let mut bindings = Vec::new();
    for result in &evidence.results {
        for participant in [
            &result.replay_result_v2.momentum,
            &result.replay_result_v2.risk,
        ] {
            let (opinion, seal) = participant
                .sealed_opinion
                .as_ref()
                .ok_or("chair_shadow_observation_missing_sealed_opinion")?;
            let expected_seal = source_bound_seal_v1(
                &opinion.opinion_id,
                &opinion.opinion_digest_v1,
                &opinion.source_result,
                &registration,
                &opinion.authority,
            )?;
            if participant.opinion_id.as_deref() != Some(opinion.opinion_id.as_str())
                || participant.seal_digest.as_deref() != Some(seal.seal_digest_v1.as_str())
                || !opinion.sealed
                || opinion.temporal_scope.prospective
                || opinion.temporal_scope.contemporaneous_claim
                || opinion.authority != OpinionAuthorityV1::historical_advisory_only()
                || seal != &expected_seal
            {
                return Err("chair_shadow_observation_opinion_or_seal_invalid".into());
            }
            let abstained = !matches!(
                participant.execution_trace.operational_shadow_result,
                JointParticipantOperationalShadowResultV2::ShadowPredictionResearchOnly
            );
            bindings.push((
                opinion.opinion_id.clone(),
                seal.seal_digest_v1.clone(),
                opinion.agent_id.clone(),
                format!("{:?}", opinion.objective),
                abstained,
            ));
        }
    }
    bindings.sort_by(|left, right| left.0.cmp(&right.0));
    if bindings.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err("chair_shadow_observation_duplicate_opinion".into());
    }
    Ok(bindings)
}

fn chair_transcripts_are_valid_v0(evidence: &ChairShadowObservationEvidenceV0) -> bool {
    let aggregate = &evidence.aggregate.replay_aggregate_v2;
    aggregate.deliberations.len() == evidence.results.len()
        && aggregate.transcript_digests.len() == aggregate.deliberations.len()
        && aggregate
            .deliberations
            .iter()
            .zip(&aggregate.transcript_digests)
            .all(|(deliberation, digest)| {
                deliberation.deliberation_version == "joint-scope-deliberation-v2"
                    && deliberation.round_count == 2
                    && deliberation.retrospective_only
                    && !deliberation.chair_observed
                    && !deliberation.vote_created
                    && !deliberation.reward_created
                    && !deliberation.penalty_created
                    && !deliberation.execution_created
                    && deliberation.transcript_digest_v2 == *digest
            })
        && aggregate
            .deliberations
            .windows(2)
            .all(|pair| pair[0].joint_scope_id < pair[1].joint_scope_id)
}

fn chair_evidence_validation_status_v0(
    evidence: &ChairShadowObservationEvidenceV0,
) -> Result<(), ChairObservationReceiptStatusV0> {
    if evidence.registration.registration_version != "joint-canonical-scope-replay-registration-v3"
        || evidence.registration.registration_digest_v3
            != joint_v3_registration_digest(&evidence.registration)
    {
        return Err(ChairObservationReceiptStatusV0::InvalidRegistration);
    }
    let (expected_aggregate, expected_ledger) =
        aggregate_joint_scope_replays_v3(&evidence.registration, &evidence.results)
            .map_err(|_| ChairObservationReceiptStatusV0::InvalidAggregate)?;
    if evidence.aggregate != expected_aggregate {
        return Err(ChairObservationReceiptStatusV0::InvalidAggregate);
    }
    if validate_joint_scope_replay_ledger_v3(&evidence.ledger).is_err()
        || evidence.ledger != expected_ledger
    {
        return Err(ChairObservationReceiptStatusV0::InvalidLedger);
    }
    if chair_source_bindings_v0(evidence).is_err() {
        return Err(ChairObservationReceiptStatusV0::InvalidOpinionSeal);
    }
    if !chair_transcripts_are_valid_v0(evidence) {
        return Err(ChairObservationReceiptStatusV0::InvalidTranscript);
    }
    Ok(())
}

pub fn chair_shadow_observation_evidence_v0(
    registration: JointCanonicalScopeReplayRegistrationV3,
    results: Vec<JointScopeReplayResultV3>,
    aggregate: JointScopeReplayAggregateV3,
    ledger: JointScopeReplayLedgerV3,
) -> Result<ChairShadowObservationEvidenceV0, String> {
    let evidence = ChairShadowObservationEvidenceV0 {
        registration,
        results,
        aggregate,
        ledger,
    };
    chair_evidence_validation_status_v0(&evidence)
        .map_err(|status| format!("chair_shadow_observation_evidence_{status:?}"))?;
    Ok(evidence)
}

pub fn chair_shadow_observation_packet_v0(
    evidence: &ChairShadowObservationEvidenceV0,
) -> Result<ChairShadowObservationPacketV0, String> {
    chair_evidence_validation_status_v0(evidence)
        .map_err(|status| format!("chair_shadow_observation_packet_{status:?}"))?;
    let bindings = chair_source_bindings_v0(evidence)?;
    let mut relationship_digests = evidence
        .aggregate
        .replay_aggregate_v2
        .deliberations
        .iter()
        .map(chair_relationship_digest_v0)
        .collect::<Vec<_>>();
    relationship_digests.sort();
    let mut transcript_digests = evidence.ledger.deliberation_transcript_digests.clone();
    transcript_digests.sort();
    let mut packet = ChairShadowObservationPacketV0 {
        packet_version: "chair-shadow-observation-packet-v0".into(),
        source_replay_version: "joint-canonical-scope-replay-v3".into(),
        source_registration_digest: evidence.registration.registration_digest_v3.clone(),
        source_ledger_digest: evidence.ledger.ledger_digest_v3.clone(),
        source_aggregate_digest: evidence.aggregate.aggregate_digest_v3.clone(),
        opinion_ids: bindings.iter().map(|binding| binding.0.clone()).collect(),
        opinion_seal_digests: bindings.iter().map(|binding| binding.1.clone()).collect(),
        relationship_digests,
        transcript_digests,
        evidence_class: ChairObservationEvidenceClassV0::RetrospectiveDevelopmentOnly,
        retrospective_only: true,
        prospective: false,
        authority: ChairShadowObservationAuthorityV0::retrospective_observation_only(),
        packet_digest: String::new(),
    };
    packet.packet_digest = chair_packet_digest_v0(&packet);
    Ok(packet)
}

fn chair_receipt_v0(
    packet: &ChairShadowObservationPacketV0,
    evidence: &ChairShadowObservationEvidenceV0,
    status: ChairObservationReceiptStatusV0,
) -> ChairShadowObservationReceiptV0 {
    let bindings = chair_source_bindings_v0(evidence).unwrap_or_default();
    let mut observed_agent_ids = bindings
        .iter()
        .map(|binding| binding.2.clone())
        .collect::<Vec<_>>();
    observed_agent_ids.sort();
    observed_agent_ids.dedup();
    let mut observed_objectives = bindings
        .iter()
        .map(|binding| binding.3.clone())
        .collect::<Vec<_>>();
    observed_objectives.sort();
    observed_objectives.dedup();
    let relationships = &evidence.aggregate.replay_aggregate_v2.relationships;
    let relationship_summary = [
        ChairObservedRelationshipCategoryV0::BothAbstained,
        ChairObservedRelationshipCategoryV0::MomentumAbstained,
        ChairObservedRelationshipCategoryV0::RiskAbstained,
        ChairObservedRelationshipCategoryV0::Tension,
        ChairObservedRelationshipCategoryV0::Orthogonal,
        ChairObservedRelationshipCategoryV0::Incomparable,
    ]
    .into_iter()
    .filter_map(|category| {
        let count = relationships
            .iter()
            .filter(|relationship| chair_relationship_category_v0(**relationship) == category)
            .count();
        (count > 0).then_some(ChairObservedRelationshipCountV0 { category, count })
    })
    .collect::<Vec<_>>();
    let scope_caveats = evidence
        .aggregate
        .replay_aggregate_v2
        .deliberations
        .iter()
        .map(|deliberation| {
            format!(
                "{}:{:?}:retrospective-two-round-observation-only",
                deliberation.joint_scope_id, deliberation.relationship
            )
        })
        .collect::<Vec<_>>();
    let observed_abstention_count = bindings.iter().filter(|binding| binding.4).count();
    let mut receipt = ChairShadowObservationReceiptV0 {
        receipt_version: "chair-shadow-observation-receipt-v0".into(),
        packet_digest: packet.packet_digest.clone(),
        status,
        observed_agent_ids,
        observed_objectives,
        observed_scope_count: evidence.results.len(),
        observed_opinion_count: bindings.len(),
        observed_abstention_count,
        relationship_summary,
        scope_caveats,
        uncertainty_flags: vec![
            ChairObservationUncertaintyV0 {
                category: ChairObservationUncertaintyCategoryV0::RetrospectiveDevelopmentOnly,
                present: true,
            },
            ChairObservationUncertaintyV0 {
                category: ChairObservationUncertaintyCategoryV0::AbstentionObserved,
                present: observed_abstention_count > 0,
            },
            ChairObservationUncertaintyV0 {
                category: ChairObservationUncertaintyCategoryV0::NoDecisionAuthority,
                present: true,
            },
            ChairObservationUncertaintyV0 {
                category: ChairObservationUncertaintyCategoryV0::NoExecutionAuthority,
                present: true,
            },
        ],
        source_aggregate_digest: evidence.aggregate.aggregate_digest_v3.clone(),
        source_ledger_digest: evidence.ledger.ledger_digest_v3.clone(),
        chair_runtime_invocations: 0,
        chair_decisions_created: 0,
        votes_created: 0,
        rewards_created: 0,
        penalties_created: 0,
        speaking_right_changes: 0,
        risk_handoffs: 0,
        executions_created: 0,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = chair_receipt_digest_v0(&receipt);
    receipt
}

fn chair_packet_validation_status_v0(
    packet: &ChairShadowObservationPacketV0,
    evidence: &ChairShadowObservationEvidenceV0,
) -> Result<(), ChairObservationReceiptStatusV0> {
    chair_evidence_validation_status_v0(evidence)?;
    if packet.prospective || !packet.retrospective_only {
        return Err(ChairObservationReceiptStatusV0::ProspectiveClaimForbidden);
    }
    if !packet.authority.is_exact_retrospective_observation_only() {
        return Err(ChairObservationReceiptStatusV0::AuthorityViolation);
    }
    let expected = chair_shadow_observation_packet_v0(evidence)
        .map_err(|_| ChairObservationReceiptStatusV0::TechnicalFailure)?;
    if packet.packet_version != expected.packet_version
        || packet.source_replay_version != expected.source_replay_version
        || packet.source_registration_digest != expected.source_registration_digest
    {
        return Err(ChairObservationReceiptStatusV0::InvalidRegistration);
    }
    if packet.source_ledger_digest != expected.source_ledger_digest {
        return Err(ChairObservationReceiptStatusV0::InvalidLedger);
    }
    if packet.source_aggregate_digest != expected.source_aggregate_digest {
        return Err(ChairObservationReceiptStatusV0::InvalidAggregate);
    }
    if packet.opinion_ids != expected.opinion_ids
        || packet.opinion_seal_digests != expected.opinion_seal_digests
    {
        return Err(ChairObservationReceiptStatusV0::InvalidOpinionSeal);
    }
    if packet.relationship_digests != expected.relationship_digests
        || packet.transcript_digests != expected.transcript_digests
    {
        return Err(ChairObservationReceiptStatusV0::InvalidTranscript);
    }
    if packet.evidence_class != expected.evidence_class
        || packet.packet_digest != chair_packet_digest_v0(packet)
        || packet.packet_digest != expected.packet_digest
    {
        return Err(ChairObservationReceiptStatusV0::TechnicalFailure);
    }
    Ok(())
}

pub fn intake_chair_shadow_observation_packet_v0(
    inbox: &mut ChairShadowObservationInboxV0,
    packet: &ChairShadowObservationPacketV0,
    evidence: &ChairShadowObservationEvidenceV0,
) -> ChairShadowObservationReceiptV0 {
    let action_counters_clear = inbox.chair_runtime_invocations == 0
        && inbox.chair_decisions_created == 0
        && inbox.votes_created == 0
        && inbox.rewards_created == 0
        && inbox.penalties_created == 0
        && inbox.speaking_right_changes == 0
        && inbox.risk_handoffs == 0
        && inbox.executions_created == 0;
    let status = if !action_counters_clear || inbox.inbox_digest != chair_inbox_digest_v0(inbox) {
        ChairObservationReceiptStatusV0::TechnicalFailure
    } else if inbox
        .packets
        .iter()
        .any(|existing| existing.packet_digest == packet.packet_digest)
        || inbox
            .rejected_packet_ids
            .iter()
            .any(|existing| existing == &packet.packet_digest)
    {
        ChairObservationReceiptStatusV0::DuplicatePacket
    } else {
        match chair_packet_validation_status_v0(packet, evidence) {
            Ok(()) => ChairObservationReceiptStatusV0::AcceptedRetrospectiveObservationOnly,
            Err(status) => status,
        }
    };
    if status == ChairObservationReceiptStatusV0::AcceptedRetrospectiveObservationOnly {
        inbox.packets.push(packet.clone());
        inbox.accepted_packet_ids.push(packet.packet_digest.clone());
    } else if !inbox
        .rejected_packet_ids
        .iter()
        .any(|existing| existing == &packet.packet_digest)
    {
        inbox.rejected_packet_ids.push(packet.packet_digest.clone());
    }
    inbox.inbox_digest = chair_inbox_digest_v0(inbox);
    chair_receipt_v0(packet, evidence, status)
}

pub fn observe_chair_shadow_observation_v0(
    evidence: &ChairShadowObservationEvidenceV0,
) -> Result<ChairShadowObservationReportV0, String> {
    let packet = chair_shadow_observation_packet_v0(evidence)?;
    let mut inbox = new_chair_shadow_observation_inbox_v0();
    let receipt = intake_chair_shadow_observation_packet_v0(&mut inbox, &packet, evidence);
    if receipt.status != ChairObservationReceiptStatusV0::AcceptedRetrospectiveObservationOnly {
        return Err(format!(
            "chair_shadow_observation_intake_{:?}",
            receipt.status
        ));
    }
    Ok(ChairShadowObservationReportV0 {
        report_version: "chair-shadow-observation-report-v0".into(),
        offline: true,
        active_committee_count: 3,
        packet,
        inbox,
        receipt,
        firewall_proof: chair_firewall_proof_v0(),
    })
}

/// Verifies a completed Shadow observation report without creating any new
/// Chair, risk, or execution state. Consumers outside this module use this
/// read-only boundary instead of reconstructing receipt or inbox digests.
pub fn validate_chair_shadow_observation_report_v0(
    report: &ChairShadowObservationReportV0,
) -> Result<(), String> {
    let expected_firewall = chair_firewall_proof_v0();
    if report.report_version != "chair-shadow-observation-report-v0"
        || !report.offline
        || report.active_committee_count != 3
        || report.packet.packet_digest != chair_packet_digest_v0(&report.packet)
        || !chair_inbox_is_valid_v0(&report.inbox)
        || report.inbox.packets.len() != 1
        || report.inbox.packets.first() != Some(&report.packet)
        || !chair_receipt_is_valid_v0(&report.receipt)
        || report.receipt.status
            != ChairObservationReceiptStatusV0::AcceptedRetrospectiveObservationOnly
        || report.receipt.packet_digest != report.packet.packet_digest
        || report.receipt.source_aggregate_digest != report.packet.source_aggregate_digest
        || report.receipt.source_ledger_digest != report.packet.source_ledger_digest
        || report.firewall_proof != expected_firewall
    {
        return Err("chair_shadow_observation_report_invalid".into());
    }
    Ok(())
}

fn chair_receipt_is_valid_v0(value: &ChairShadowObservationReceiptV0) -> bool {
    value.receipt_version == "chair-shadow-observation-receipt-v0"
        && value.chair_runtime_invocations == 0
        && value.chair_decisions_created == 0
        && value.votes_created == 0
        && value.rewards_created == 0
        && value.penalties_created == 0
        && value.speaking_right_changes == 0
        && value.risk_handoffs == 0
        && value.executions_created == 0
        && value.receipt_digest == chair_receipt_digest_v0(value)
}

fn chair_inbox_is_valid_v0(value: &ChairShadowObservationInboxV0) -> bool {
    value.inbox_version == "chair-shadow-observation-inbox-v0"
        && value.chair_runtime_invocations == 0
        && value.chair_decisions_created == 0
        && value.votes_created == 0
        && value.rewards_created == 0
        && value.penalties_created == 0
        && value.speaking_right_changes == 0
        && value.risk_handoffs == 0
        && value.executions_created == 0
        && value
            .packets
            .iter()
            .all(|packet| packet.packet_digest == chair_packet_digest_v0(packet))
        && value.accepted_packet_ids
            == value
                .packets
                .iter()
                .map(|packet| packet.packet_digest.clone())
                .collect::<Vec<_>>()
        && value.inbox_digest == chair_inbox_digest_v0(value)
}

pub fn validate_chair_shadow_observation_storage_v0(
    storage: &ChairShadowObservationStorageV0,
) -> Result<(), String> {
    if storage.storage_version != "chair-shadow-observation-storage-v0"
        || !chair_inbox_is_valid_v0(&storage.inbox)
        || storage.receipts.len() != storage.inbox.packets.len()
        || storage.firewall_proofs.len() != storage.inbox.packets.len()
        || storage
            .receipts
            .iter()
            .zip(&storage.inbox.packets)
            .any(|(receipt, packet)| {
                !chair_receipt_is_valid_v0(receipt)
                    || receipt.status
                        != ChairObservationReceiptStatusV0::AcceptedRetrospectiveObservationOnly
                    || receipt.packet_digest != packet.packet_digest
            })
        || storage
            .firewall_proofs
            .iter()
            .any(|proof| proof != &chair_firewall_proof_v0())
        || storage.storage_digest != chair_storage_digest_v0(storage)
    {
        return Err("chair_shadow_observation_storage_invalid".into());
    }
    Ok(())
}

fn new_chair_shadow_observation_storage_v0() -> ChairShadowObservationStorageV0 {
    let mut storage = ChairShadowObservationStorageV0 {
        storage_version: "chair-shadow-observation-storage-v0".into(),
        inbox: new_chair_shadow_observation_inbox_v0(),
        receipts: vec![],
        firewall_proofs: vec![],
        storage_digest: String::new(),
    };
    storage.storage_digest = chair_storage_digest_v0(&storage);
    storage
}

fn write_chair_shadow_observation_storage_v0(
    path: &Path,
    storage: &ChairShadowObservationStorageV0,
) -> Result<(), String> {
    validate_chair_shadow_observation_storage_v0(storage)?;
    let parent = path
        .parent()
        .ok_or_else(|| "chair_shadow_observation_storage_path_invalid".to_string())?;
    fs::create_dir_all(parent).map_err(|_| "chair_shadow_observation_storage_write".to_string())?;
    let encoded = serde_json::to_vec(storage)
        .map_err(|_| "chair_shadow_observation_storage_write".to_string())?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, encoded)
        .map_err(|_| "chair_shadow_observation_storage_write".to_string())?;
    fs::rename(&temporary, path).map_err(|_| "chair_shadow_observation_storage_write".to_string())
}

pub fn read_chair_shadow_observation_storage_v0(
    path: &Path,
) -> Result<ChairShadowObservationStorageV0, String> {
    let encoded =
        fs::read(path).map_err(|_| "chair_shadow_observation_storage_read".to_string())?;
    let storage = serde_json::from_slice(&encoded)
        .map_err(|_| "chair_shadow_observation_storage_read".to_string())?;
    validate_chair_shadow_observation_storage_v0(&storage)?;
    Ok(storage)
}

pub fn append_chair_shadow_observation_storage_v0(
    path: &Path,
    report: &ChairShadowObservationReportV0,
) -> Result<ChairShadowObservationStorageV0, String> {
    if !report.offline
        || report.active_committee_count != 3
        || !chair_inbox_is_valid_v0(&report.inbox)
        || !chair_receipt_is_valid_v0(&report.receipt)
        || report.receipt.status
            != ChairObservationReceiptStatusV0::AcceptedRetrospectiveObservationOnly
        || report.receipt.packet_digest != report.packet.packet_digest
        || report.firewall_proof != chair_firewall_proof_v0()
    {
        return Err("chair_shadow_observation_storage_report_invalid".into());
    }
    let mut storage = if path.exists() {
        read_chair_shadow_observation_storage_v0(path)?
    } else {
        new_chair_shadow_observation_storage_v0()
    };
    if let Some(index) = storage
        .inbox
        .packets
        .iter()
        .position(|packet| packet.packet_digest == report.packet.packet_digest)
    {
        if storage.receipts.get(index) == Some(&report.receipt)
            && storage.firewall_proofs.get(index) == Some(&report.firewall_proof)
        {
            return Ok(storage);
        }
        return Err("chair_shadow_observation_storage_duplicate_conflict".into());
    }
    storage.inbox.packets.push(report.packet.clone());
    storage
        .inbox
        .accepted_packet_ids
        .push(report.packet.packet_digest.clone());
    storage.inbox.inbox_digest = chair_inbox_digest_v0(&storage.inbox);
    storage.receipts.push(report.receipt.clone());
    storage.firewall_proofs.push(report.firewall_proof.clone());
    storage.storage_digest = chair_storage_digest_v0(&storage);
    write_chair_shadow_observation_storage_v0(path, &storage)?;
    let reopened = read_chair_shadow_observation_storage_v0(path)?;
    if reopened != storage {
        return Err("chair_shadow_observation_storage_reopen_mismatch".into());
    }
    Ok(reopened)
}

/// The sealed, objective-specific contract that a future learned-agent event must
/// satisfy before it can be considered by the reward bridge.  It deliberately
/// contains only identities and pre-registered policy references: no label,
/// probability, owner note, or Chair state can enter through this boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedProspectiveContractV0 {
    pub objective: LearnedAgentObjectiveV0,
    pub agent_id: String,
    pub challenge_digest: String,
    pub model_artifact_digest: String,
    pub prediction_horizon_digest: String,
    pub cutoff_exclusive_timestamp: u64,
    pub sealed_shadow_only: bool,
    pub contract_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedRewardSampleGateV0 {
    pub minimum_mature_events: usize,
    pub minimum_support_qualified_events: usize,
    pub minimum_regime_coverage: usize,
    pub maximum_integrity_failures: usize,
    pub gate_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedRewardEligibilityRegistrationInputV0 {
    pub momentum: LearnedProspectiveContractV0,
    pub cycle_risk: LearnedProspectiveContractV0,
    pub attribution_policy_digest: String,
    pub maturity_policy_digest: String,
    pub sample_gate_policy_digest: String,
    pub objective_mapping_policy_digest: String,
    pub integrity_policy_digest: String,
}

/// Immutable Phase-A registration.  This object intentionally has no route to
/// `apply_chair_reward_penalty`, voice state, tiers, cooldowns, or execution.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedRewardEligibilityRegistrationV0 {
    pub registration_version: String,
    pub momentum_challenge_digest: String,
    pub risk_tournament_digest: String,
    pub attribution_policy_digest: String,
    pub maturity_policy_digest: String,
    pub sample_gate_policy_digest: String,
    pub objective_mapping_policy_digest: String,
    pub integrity_policy_digest: String,
    pub retrospective_evidence_forbidden: bool,
    pub owner_input_forbidden: bool,
    pub interim_metrics_forbidden: bool,
    pub one_time_opening_required: bool,
    pub finalized_outcome_required: bool,
    pub reward_application_forbidden: bool,
    pub voice_mutation_forbidden: bool,
    pub cooldown_mutation_forbidden: bool,
    pub promotion_mutation_forbidden: bool,
    pub registration_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearnedOutcomeMaturityStatusV0 {
    AwaitingMaturity,
    MatureUnopened,
    MatureOpenedOnce,
    OpenedEarlyInvalid,
    DuplicateOpeningInvalid,
    IntegrityInvalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedOutcomeOpeningRequestV0 {
    pub event_timestamp: u64,
    pub maturity_timestamp: u64,
    pub observed_timestamp: u64,
    pub required_finalized_rows_present: bool,
    pub event_identity_matches: bool,
    pub challenge_valid: bool,
    pub explicit_authorization: bool,
    pub already_opened: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedProspectiveEventAttributionV0 {
    pub attribution_version: String,
    pub event_id: String,
    pub event_digest: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub opinion_id: String,
    pub opinion_digest: String,
    pub opinion_seal_digest: String,
    pub challenge_digest: String,
    pub model_artifact_digest: String,
    pub raw_evidence_digest: String,
    pub event_timestamp: u64,
    pub maturity_timestamp: u64,
    pub prediction_horizon_digest: String,
    pub support_status_digest: String,
    pub attribution_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearnedAbstentionAttributionV0 {
    JustifiedCapitalProtection,
    CorrectUncertainty,
    MissedMaterialOpportunity,
    FailedToWarnMaterialRisk,
    NeutralUninformative,
    NotYetEvaluable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MomentumProspectiveOutcomeV0 {
    pub directional_label_correct: bool,
    pub support_qualified_brier_improved: bool,
    pub calibration_improved: bool,
    pub high_confidence_error: bool,
    pub baseline_beaten: bool,
    pub abstention: LearnedAbstentionAttributionV0,
    pub probability_collapse: bool,
    pub support_qualified: bool,
    pub regime_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleRiskProspectiveOutcomeV0 {
    pub downside_label_correct: bool,
    pub support_qualified_brier_improved: bool,
    pub calibration_improved: bool,
    pub high_confidence_false_negative: bool,
    pub correct_elevated_risk_warning: bool,
    pub false_permanent_alarm: bool,
    pub abstention: LearnedAbstentionAttributionV0,
    pub probability_collapse: bool,
    pub support_qualified: bool,
    pub regime_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearnedProspectiveOutcomePayloadV0 {
    Momentum(MomentumProspectiveOutcomeV0),
    CycleRisk(CycleRiskProspectiveOutcomeV0),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearnedRewardEligibilityStatusV0 {
    EligibleForCandidateComputation,
    IneligibleNoProspectiveOutcomes,
    IneligibleAwaitingMaturity,
    IneligibleMinimumSamples,
    IneligibleInsufficientRegimeCoverage,
    IneligibleEarlyLabelAccess,
    IneligibleIntegrityFailure,
    IneligibleChallengeInvalidated,
    IneligibleUnsupportedObjective,
    IneligibleRetrospectiveEvidence,
    IneligibleOwnerInfluence,
    IneligibleDuplicateOpening,
    TechnicalFailure,
}

fn learned_contract_digest_v0(contract: &LearnedProspectiveContractV0) -> String {
    digest(&(
        "learned-prospective-contract-v0",
        contract.objective,
        &contract.agent_id,
        &contract.challenge_digest,
        &contract.model_artifact_digest,
        &contract.prediction_horizon_digest,
        contract.cutoff_exclusive_timestamp,
        contract.sealed_shadow_only,
    ))
}

fn learned_reward_gate_digest_v0(gate: &LearnedRewardSampleGateV0) -> String {
    digest(&(
        "learned-reward-sample-gate-v0",
        gate.minimum_mature_events,
        gate.minimum_support_qualified_events,
        gate.minimum_regime_coverage,
        gate.maximum_integrity_failures,
    ))
}

pub fn new_learned_prospective_contract_v0(
    objective: LearnedAgentObjectiveV0,
    challenge_digest: String,
    model_artifact_digest: String,
    prediction_horizon_digest: String,
    cutoff_exclusive_timestamp: u64,
) -> Result<LearnedProspectiveContractV0, String> {
    let agent_id = match objective {
        LearnedAgentObjectiveV0::DirectionalMomentum => MOMENTUM_AGENT_ID_V0,
        LearnedAgentObjectiveV0::DownsideRisk => CYCLE_RISK_SHADOW_AGENT_ID_V0,
    };
    let mut contract = LearnedProspectiveContractV0 {
        objective,
        agent_id: agent_id.into(),
        challenge_digest,
        model_artifact_digest,
        prediction_horizon_digest,
        cutoff_exclusive_timestamp,
        sealed_shadow_only: true,
        contract_digest: String::new(),
    };
    contract.contract_digest = learned_contract_digest_v0(&contract);
    validate_learned_prospective_contract_v0(&contract)?;
    Ok(contract)
}

pub fn new_learned_reward_sample_gate_v0(
    minimum_mature_events: usize,
    minimum_support_qualified_events: usize,
    minimum_regime_coverage: usize,
) -> Result<LearnedRewardSampleGateV0, String> {
    let mut gate = LearnedRewardSampleGateV0 {
        minimum_mature_events,
        minimum_support_qualified_events,
        minimum_regime_coverage,
        maximum_integrity_failures: 0,
        gate_digest: String::new(),
    };
    gate.gate_digest = learned_reward_gate_digest_v0(&gate);
    validate_learned_reward_sample_gate_v0(&gate)?;
    Ok(gate)
}

fn learned_reward_registration_digest_v0(
    registration: &LearnedRewardEligibilityRegistrationV0,
) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        registration.registration_version,
        registration.momentum_challenge_digest,
        registration.risk_tournament_digest,
        registration.attribution_policy_digest,
        registration.maturity_policy_digest,
        registration.sample_gate_policy_digest,
        registration.objective_mapping_policy_digest,
        registration.integrity_policy_digest,
        registration.retrospective_evidence_forbidden,
        registration.owner_input_forbidden,
        registration.interim_metrics_forbidden,
        registration.one_time_opening_required,
        registration.finalized_outcome_required,
        registration.reward_application_forbidden,
        registration.voice_mutation_forbidden,
        registration.cooldown_mutation_forbidden,
        registration.promotion_mutation_forbidden,
    ))
}

fn learned_attribution_digest_v0(event: &LearnedProspectiveEventAttributionV0) -> String {
    stable_hash_string(&format!(
        "{:?}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        event.attribution_version,
        event.event_id,
        event.event_digest,
        event.agent_id,
        event.objective,
        event.opinion_id,
        event.opinion_digest,
        event.opinion_seal_digest,
        event.challenge_digest,
        event.model_artifact_digest,
        event.raw_evidence_digest,
        event.event_timestamp,
        event.maturity_timestamp,
        event.prediction_horizon_digest,
        event.support_status_digest,
    ))
}

pub fn validate_learned_prospective_contract_v0(
    contract: &LearnedProspectiveContractV0,
) -> Result<(), String> {
    let expected_agent = match contract.objective {
        LearnedAgentObjectiveV0::DirectionalMomentum => MOMENTUM_AGENT_ID_V0,
        LearnedAgentObjectiveV0::DownsideRisk => CYCLE_RISK_SHADOW_AGENT_ID_V0,
    };
    if contract.agent_id != expected_agent
        || !contract.sealed_shadow_only
        || contract.cutoff_exclusive_timestamp == 0
        || [
            &contract.challenge_digest,
            &contract.model_artifact_digest,
            &contract.prediction_horizon_digest,
        ]
        .iter()
        .any(|value| value.is_empty())
        || contract.contract_digest != learned_contract_digest_v0(contract)
    {
        Err("learned_prospective_contract_invalid".into())
    } else {
        Ok(())
    }
}

pub fn validate_learned_reward_sample_gate_v0(
    gate: &LearnedRewardSampleGateV0,
) -> Result<(), String> {
    if gate.minimum_mature_events == 0
        || gate.minimum_support_qualified_events == 0
        || gate.minimum_regime_coverage == 0
        || gate.maximum_integrity_failures != 0
        || gate.gate_digest != learned_reward_gate_digest_v0(gate)
    {
        Err("learned_reward_sample_gate_invalid".into())
    } else {
        Ok(())
    }
}

pub fn pre_register_learned_reward_eligibility_v0(
    input: &LearnedRewardEligibilityRegistrationInputV0,
) -> Result<LearnedRewardEligibilityRegistrationV0, String> {
    validate_learned_prospective_contract_v0(&input.momentum)?;
    validate_learned_prospective_contract_v0(&input.cycle_risk)?;
    if input.momentum.objective != LearnedAgentObjectiveV0::DirectionalMomentum
        || input.cycle_risk.objective != LearnedAgentObjectiveV0::DownsideRisk
        || [
            &input.attribution_policy_digest,
            &input.maturity_policy_digest,
            &input.sample_gate_policy_digest,
            &input.objective_mapping_policy_digest,
            &input.integrity_policy_digest,
        ]
        .iter()
        .any(|value| value.is_empty())
    {
        return Err("learned_reward_registration_input_invalid".into());
    }
    let mut registration = LearnedRewardEligibilityRegistrationV0 {
        registration_version: "learned-reward-eligibility-registration-v0".into(),
        momentum_challenge_digest: input.momentum.challenge_digest.clone(),
        risk_tournament_digest: input.cycle_risk.challenge_digest.clone(),
        attribution_policy_digest: input.attribution_policy_digest.clone(),
        maturity_policy_digest: input.maturity_policy_digest.clone(),
        sample_gate_policy_digest: input.sample_gate_policy_digest.clone(),
        objective_mapping_policy_digest: input.objective_mapping_policy_digest.clone(),
        integrity_policy_digest: input.integrity_policy_digest.clone(),
        retrospective_evidence_forbidden: true,
        owner_input_forbidden: true,
        interim_metrics_forbidden: true,
        one_time_opening_required: true,
        finalized_outcome_required: true,
        reward_application_forbidden: true,
        voice_mutation_forbidden: true,
        cooldown_mutation_forbidden: true,
        promotion_mutation_forbidden: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = learned_reward_registration_digest_v0(&registration);
    validate_learned_reward_eligibility_registration_v0(&registration)?;
    Ok(registration)
}

pub fn validate_learned_reward_eligibility_registration_v0(
    registration: &LearnedRewardEligibilityRegistrationV0,
) -> Result<(), String> {
    if registration.registration_version != "learned-reward-eligibility-registration-v0"
        || [
            &registration.momentum_challenge_digest,
            &registration.risk_tournament_digest,
            &registration.attribution_policy_digest,
            &registration.maturity_policy_digest,
            &registration.sample_gate_policy_digest,
            &registration.objective_mapping_policy_digest,
            &registration.integrity_policy_digest,
        ]
        .iter()
        .any(|value| value.is_empty())
        || !registration.retrospective_evidence_forbidden
        || !registration.owner_input_forbidden
        || !registration.interim_metrics_forbidden
        || !registration.one_time_opening_required
        || !registration.finalized_outcome_required
        || !registration.reward_application_forbidden
        || !registration.voice_mutation_forbidden
        || !registration.cooldown_mutation_forbidden
        || !registration.promotion_mutation_forbidden
        || registration.registration_digest != learned_reward_registration_digest_v0(registration)
    {
        Err("learned_reward_registration_invalid".into())
    } else {
        Ok(())
    }
}

pub fn learned_outcome_maturity_status_v0(
    request: &LearnedOutcomeOpeningRequestV0,
) -> LearnedOutcomeMaturityStatusV0 {
    if !request.required_finalized_rows_present
        || !request.event_identity_matches
        || !request.challenge_valid
        || !request.explicit_authorization
        || request.event_timestamp == 0
        || request.maturity_timestamp <= request.event_timestamp
    {
        LearnedOutcomeMaturityStatusV0::IntegrityInvalid
    } else if request.already_opened {
        LearnedOutcomeMaturityStatusV0::DuplicateOpeningInvalid
    } else if request.observed_timestamp < request.maturity_timestamp {
        LearnedOutcomeMaturityStatusV0::OpenedEarlyInvalid
    } else if request.observed_timestamp == request.maturity_timestamp {
        LearnedOutcomeMaturityStatusV0::MatureUnopened
    } else {
        LearnedOutcomeMaturityStatusV0::MatureOpenedOnce
    }
}

pub fn validate_learned_prospective_event_attribution_v0(
    event: &LearnedProspectiveEventAttributionV0,
    contract: &LearnedProspectiveContractV0,
    known_event_ids: &BTreeSet<String>,
) -> Result<(), String> {
    validate_learned_prospective_contract_v0(contract)?;
    if event.attribution_version != "learned-prospective-event-attribution-v0"
        || known_event_ids.contains(&event.event_id)
        || event.agent_id != contract.agent_id
        || event.objective != contract.objective
        || event.challenge_digest != contract.challenge_digest
        || event.model_artifact_digest != contract.model_artifact_digest
        || event.prediction_horizon_digest != contract.prediction_horizon_digest
        || event.event_timestamp <= contract.cutoff_exclusive_timestamp
        || event.maturity_timestamp <= event.event_timestamp
        || [
            &event.event_id,
            &event.event_digest,
            &event.opinion_id,
            &event.opinion_digest,
            &event.opinion_seal_digest,
            &event.raw_evidence_digest,
            &event.support_status_digest,
        ]
        .iter()
        .any(|value| value.is_empty())
        || event.attribution_digest != learned_attribution_digest_v0(event)
    {
        Err("learned_prospective_event_attribution_invalid".into())
    } else {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedProspectiveOutcomeRecordV0 {
    pub record_version: String,
    pub event_id: String,
    pub attribution_digest: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub payload: LearnedProspectiveOutcomePayloadV0,
    pub maturity_status: LearnedOutcomeMaturityStatusV0,
    pub challenge_valid: bool,
    pub integrity_valid: bool,
    pub retrospective_evidence: bool,
    pub owner_influence: bool,
    pub outcome_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedRewardEligibilityRecordV0 {
    pub record_version: String,
    pub registration_digest: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub prospective_event_count: usize,
    pub mature_outcome_count: usize,
    pub support_qualified_count: usize,
    pub regime_coverage_count: usize,
    pub eligibility_status: LearnedRewardEligibilityStatusV0,
    pub eligibility_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedProspectiveOutcomeLedgerV0 {
    pub ledger_version: String,
    pub registration_digest: String,
    pub event_attributions: Vec<LearnedProspectiveEventAttributionV0>,
    pub matured_outcomes: Vec<LearnedProspectiveOutcomeRecordV0>,
    pub eligibility_records: Vec<LearnedRewardEligibilityRecordV0>,
    pub label_open_count: usize,
    pub reward_candidate_count: usize,
    pub reward_apply_count: usize,
    pub ledger_digest: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LearnedRewardSignalV0 {
    CalibratedProspectiveAccuracy,
    CorrectRiskWarning,
    JustifiedCapitalProtection,
    UsefulIndependentContribution,
    DoctrineConsistentAbstention,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LearnedPenaltySignalV0 {
    HighConfidenceProspectiveError,
    HighConfidenceRiskFalseNegative,
    ProbabilityCollapse,
    RepeatedOutOfSupportAssertion,
    DoctrineViolation,
    CorrelatedNonIndependentContribution,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedRewardInputCandidateV0 {
    pub candidate_version: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub prospective_outcome_ledger_digest: String,
    pub eligibility_record_digest: String,
    pub typed_reward_signals: Vec<LearnedRewardSignalV0>,
    pub typed_penalty_signals: Vec<LearnedPenaltySignalV0>,
    pub eligible_for_existing_reward_compute: bool,
    pub eligible_for_application: bool,
    pub candidate_digest: String,
}

fn learned_outcome_digest_v0(record: &LearnedProspectiveOutcomeRecordV0) -> String {
    digest(&(
        &record.record_version,
        &record.event_id,
        &record.attribution_digest,
        &record.agent_id,
        record.objective,
        &record.payload,
        record.maturity_status,
        record.challenge_valid,
        record.integrity_valid,
        record.retrospective_evidence,
        record.owner_influence,
    ))
}

fn learned_eligibility_digest_v0(record: &LearnedRewardEligibilityRecordV0) -> String {
    digest(&(
        &record.record_version,
        &record.registration_digest,
        &record.agent_id,
        record.objective,
        record.prospective_event_count,
        record.mature_outcome_count,
        record.support_qualified_count,
        record.regime_coverage_count,
        record.eligibility_status,
    ))
}

fn learned_outcome_ledger_digest_v0(ledger: &LearnedProspectiveOutcomeLedgerV0) -> String {
    digest(&(
        &ledger.ledger_version,
        &ledger.registration_digest,
        &ledger.event_attributions,
        &ledger.matured_outcomes,
        &ledger.eligibility_records,
        ledger.label_open_count,
        ledger.reward_candidate_count,
        ledger.reward_apply_count,
    ))
}

fn learned_reward_candidate_digest_v0(candidate: &LearnedRewardInputCandidateV0) -> String {
    digest(&(
        &candidate.candidate_version,
        &candidate.agent_id,
        candidate.objective,
        &candidate.prospective_outcome_ledger_digest,
        &candidate.eligibility_record_digest,
        &candidate.typed_reward_signals,
        &candidate.typed_penalty_signals,
        candidate.eligible_for_existing_reward_compute,
        candidate.eligible_for_application,
    ))
}

pub fn new_learned_prospective_outcome_ledger_v0(
    registration: &LearnedRewardEligibilityRegistrationV0,
) -> Result<LearnedProspectiveOutcomeLedgerV0, String> {
    validate_learned_reward_eligibility_registration_v0(registration)?;
    let mut ledger = LearnedProspectiveOutcomeLedgerV0 {
        ledger_version: "learned-prospective-outcome-ledger-v0".into(),
        registration_digest: registration.registration_digest.clone(),
        event_attributions: vec![],
        matured_outcomes: vec![],
        eligibility_records: vec![],
        label_open_count: 0,
        reward_candidate_count: 0,
        reward_apply_count: 0,
        ledger_digest: String::new(),
    };
    ledger.ledger_digest = learned_outcome_ledger_digest_v0(&ledger);
    Ok(ledger)
}

pub fn validate_learned_prospective_outcome_ledger_v0(
    ledger: &LearnedProspectiveOutcomeLedgerV0,
    registration: &LearnedRewardEligibilityRegistrationV0,
) -> Result<(), String> {
    validate_learned_reward_eligibility_registration_v0(registration)?;
    let event_ids = ledger
        .event_attributions
        .iter()
        .map(|event| event.event_id.clone())
        .collect::<BTreeSet<_>>();
    if ledger.ledger_version != "learned-prospective-outcome-ledger-v0"
        || ledger.registration_digest != registration.registration_digest
        || event_ids.len() != ledger.event_attributions.len()
        || ledger.matured_outcomes.iter().any(|record| {
            !event_ids.contains(&record.event_id)
                || record.outcome_digest != learned_outcome_digest_v0(record)
        })
        || ledger
            .eligibility_records
            .iter()
            .any(|record| record.eligibility_digest != learned_eligibility_digest_v0(record))
        || ledger.reward_apply_count != 0
        || ledger.ledger_digest != learned_outcome_ledger_digest_v0(ledger)
    {
        Err("learned_prospective_outcome_ledger_invalid".into())
    } else {
        Ok(())
    }
}

pub fn append_learned_prospective_event_attribution_v0(
    ledger: &mut LearnedProspectiveOutcomeLedgerV0,
    registration: &LearnedRewardEligibilityRegistrationV0,
    event: LearnedProspectiveEventAttributionV0,
    contract: &LearnedProspectiveContractV0,
) -> Result<(), String> {
    validate_learned_prospective_outcome_ledger_v0(ledger, registration)?;
    let known = ledger
        .event_attributions
        .iter()
        .map(|existing| existing.event_id.clone())
        .collect::<BTreeSet<_>>();
    validate_learned_prospective_event_attribution_v0(&event, contract, &known)?;
    ledger.event_attributions.push(event);
    ledger
        .event_attributions
        .sort_by(|left, right| left.event_id.cmp(&right.event_id));
    ledger.ledger_digest = learned_outcome_ledger_digest_v0(ledger);
    Ok(())
}

fn outcome_payload_matches_objective_v0(
    objective: LearnedAgentObjectiveV0,
    payload: &LearnedProspectiveOutcomePayloadV0,
) -> bool {
    matches!(
        (objective, payload),
        (
            LearnedAgentObjectiveV0::DirectionalMomentum,
            LearnedProspectiveOutcomePayloadV0::Momentum(_)
        ) | (
            LearnedAgentObjectiveV0::DownsideRisk,
            LearnedProspectiveOutcomePayloadV0::CycleRisk(_)
        )
    )
}

pub fn append_learned_matured_outcome_v0(
    ledger: &mut LearnedProspectiveOutcomeLedgerV0,
    registration: &LearnedRewardEligibilityRegistrationV0,
    mut record: LearnedProspectiveOutcomeRecordV0,
) -> Result<(), String> {
    validate_learned_prospective_outcome_ledger_v0(ledger, registration)?;
    let event = ledger
        .event_attributions
        .iter()
        .find(|event| event.event_id == record.event_id)
        .ok_or("learned_matured_outcome_event_missing")?;
    if ledger.label_open_count != 0
        || ledger
            .matured_outcomes
            .iter()
            .any(|existing| existing.event_id == record.event_id)
        || event.attribution_digest != record.attribution_digest
        || event.agent_id != record.agent_id
        || event.objective != record.objective
        || record.maturity_status != LearnedOutcomeMaturityStatusV0::MatureOpenedOnce
        || !outcome_payload_matches_objective_v0(record.objective, &record.payload)
        || !record.challenge_valid
        || !record.integrity_valid
        || record.retrospective_evidence
        || record.owner_influence
    {
        return Err("learned_matured_outcome_invalid_or_duplicate_opening".into());
    }
    record.record_version = "learned-prospective-outcome-record-v0".into();
    record.outcome_digest = learned_outcome_digest_v0(&record);
    ledger.matured_outcomes.push(record);
    ledger.label_open_count = 1;
    ledger.ledger_digest = learned_outcome_ledger_digest_v0(ledger);
    Ok(())
}

fn outcome_support_and_regime_v0(record: &LearnedProspectiveOutcomeRecordV0) -> (bool, String) {
    match &record.payload {
        LearnedProspectiveOutcomePayloadV0::Momentum(outcome) => {
            (outcome.support_qualified, outcome.regime_digest.clone())
        }
        LearnedProspectiveOutcomePayloadV0::CycleRisk(outcome) => {
            (outcome.support_qualified, outcome.regime_digest.clone())
        }
    }
}

pub fn derive_learned_reward_eligibility_v0(
    registration: &LearnedRewardEligibilityRegistrationV0,
    gate: &LearnedRewardSampleGateV0,
    ledger: &LearnedProspectiveOutcomeLedgerV0,
    objective: LearnedAgentObjectiveV0,
) -> Result<LearnedRewardEligibilityRecordV0, String> {
    validate_learned_reward_eligibility_registration_v0(registration)?;
    validate_learned_reward_sample_gate_v0(gate)?;
    validate_learned_prospective_outcome_ledger_v0(ledger, registration)?;
    let agent_id = match objective {
        LearnedAgentObjectiveV0::DirectionalMomentum => MOMENTUM_AGENT_ID_V0,
        LearnedAgentObjectiveV0::DownsideRisk => CYCLE_RISK_SHADOW_AGENT_ID_V0,
    };
    let events = ledger
        .event_attributions
        .iter()
        .filter(|event| event.objective == objective)
        .collect::<Vec<_>>();
    let outcomes = ledger
        .matured_outcomes
        .iter()
        .filter(|outcome| outcome.objective == objective)
        .collect::<Vec<_>>();
    let (support_qualified_count, regime_coverage_count) = outcomes.iter().fold(
        (0usize, BTreeSet::new()),
        |(support_count, mut regimes), outcome| {
            let (support, regime) = outcome_support_and_regime_v0(outcome);
            if support {
                regimes.insert(regime);
            }
            (support_count + usize::from(support), regimes)
        },
    );
    let eligibility_status = if events.is_empty() {
        LearnedRewardEligibilityStatusV0::IneligibleNoProspectiveOutcomes
    } else if outcomes.iter().any(|outcome| {
        outcome.maturity_status == LearnedOutcomeMaturityStatusV0::OpenedEarlyInvalid
    }) {
        LearnedRewardEligibilityStatusV0::IneligibleEarlyLabelAccess
    } else if outcomes.iter().any(|outcome| {
        outcome.maturity_status == LearnedOutcomeMaturityStatusV0::DuplicateOpeningInvalid
    }) {
        LearnedRewardEligibilityStatusV0::IneligibleDuplicateOpening
    } else if outcomes
        .iter()
        .any(|outcome| outcome.retrospective_evidence)
    {
        LearnedRewardEligibilityStatusV0::IneligibleRetrospectiveEvidence
    } else if outcomes.iter().any(|outcome| outcome.owner_influence) {
        LearnedRewardEligibilityStatusV0::IneligibleOwnerInfluence
    } else if outcomes.iter().any(|outcome| !outcome.challenge_valid) {
        LearnedRewardEligibilityStatusV0::IneligibleChallengeInvalidated
    } else if outcomes.iter().any(|outcome| {
        !outcome.integrity_valid
            || outcome.maturity_status == LearnedOutcomeMaturityStatusV0::IntegrityInvalid
    }) {
        LearnedRewardEligibilityStatusV0::IneligibleIntegrityFailure
    } else if outcomes
        .iter()
        .any(|outcome| !outcome_payload_matches_objective_v0(objective, &outcome.payload))
    {
        LearnedRewardEligibilityStatusV0::IneligibleUnsupportedObjective
    } else if outcomes.len() < events.len() {
        LearnedRewardEligibilityStatusV0::IneligibleAwaitingMaturity
    } else if outcomes.len() < gate.minimum_mature_events
        || support_qualified_count < gate.minimum_support_qualified_events
    {
        LearnedRewardEligibilityStatusV0::IneligibleMinimumSamples
    } else if regime_coverage_count.len() < gate.minimum_regime_coverage {
        LearnedRewardEligibilityStatusV0::IneligibleInsufficientRegimeCoverage
    } else {
        LearnedRewardEligibilityStatusV0::EligibleForCandidateComputation
    };
    let mut record = LearnedRewardEligibilityRecordV0 {
        record_version: "learned-reward-eligibility-record-v0".into(),
        registration_digest: registration.registration_digest.clone(),
        agent_id: agent_id.into(),
        objective,
        prospective_event_count: events.len(),
        mature_outcome_count: outcomes.len(),
        support_qualified_count,
        regime_coverage_count: regime_coverage_count.len(),
        eligibility_status,
        eligibility_digest: String::new(),
    };
    record.eligibility_digest = learned_eligibility_digest_v0(&record);
    Ok(record)
}

pub fn learned_reward_input_candidate_v0(
    eligibility: &LearnedRewardEligibilityRecordV0,
    ledger: &LearnedProspectiveOutcomeLedgerV0,
) -> Result<LearnedRewardInputCandidateV0, String> {
    if eligibility.eligibility_digest != learned_eligibility_digest_v0(eligibility)
        || eligibility.eligibility_status
            != LearnedRewardEligibilityStatusV0::EligibleForCandidateComputation
    {
        return Err("learned_reward_candidate_ineligible".into());
    }
    let mut rewards = BTreeSet::new();
    let mut penalties = BTreeSet::new();
    for outcome in ledger
        .matured_outcomes
        .iter()
        .filter(|outcome| outcome.objective == eligibility.objective)
    {
        match &outcome.payload {
            LearnedProspectiveOutcomePayloadV0::Momentum(value) => {
                if value.directional_label_correct && value.support_qualified_brier_improved {
                    rewards.insert(LearnedRewardSignalV0::CalibratedProspectiveAccuracy);
                }
                if value.baseline_beaten && value.support_qualified {
                    rewards.insert(LearnedRewardSignalV0::UsefulIndependentContribution);
                }
                if matches!(
                    value.abstention,
                    LearnedAbstentionAttributionV0::JustifiedCapitalProtection
                ) {
                    rewards.insert(LearnedRewardSignalV0::JustifiedCapitalProtection);
                    rewards.insert(LearnedRewardSignalV0::DoctrineConsistentAbstention);
                }
                if value.high_confidence_error {
                    penalties.insert(LearnedPenaltySignalV0::HighConfidenceProspectiveError);
                }
                if value.probability_collapse {
                    penalties.insert(LearnedPenaltySignalV0::ProbabilityCollapse);
                }
            }
            LearnedProspectiveOutcomePayloadV0::CycleRisk(value) => {
                if value.downside_label_correct && value.support_qualified_brier_improved {
                    rewards.insert(LearnedRewardSignalV0::CalibratedProspectiveAccuracy);
                }
                if value.correct_elevated_risk_warning {
                    rewards.insert(LearnedRewardSignalV0::CorrectRiskWarning);
                }
                if matches!(
                    value.abstention,
                    LearnedAbstentionAttributionV0::JustifiedCapitalProtection
                ) {
                    rewards.insert(LearnedRewardSignalV0::JustifiedCapitalProtection);
                    rewards.insert(LearnedRewardSignalV0::DoctrineConsistentAbstention);
                }
                if value.high_confidence_false_negative {
                    penalties.insert(LearnedPenaltySignalV0::HighConfidenceRiskFalseNegative);
                }
                if value.probability_collapse {
                    penalties.insert(LearnedPenaltySignalV0::ProbabilityCollapse);
                }
            }
        }
    }
    let mut candidate = LearnedRewardInputCandidateV0 {
        candidate_version: "learned-reward-input-candidate-v0".into(),
        agent_id: eligibility.agent_id.clone(),
        objective: eligibility.objective,
        prospective_outcome_ledger_digest: ledger.ledger_digest.clone(),
        eligibility_record_digest: eligibility.eligibility_digest.clone(),
        typed_reward_signals: rewards.into_iter().collect(),
        typed_penalty_signals: penalties.into_iter().collect(),
        eligible_for_existing_reward_compute: true,
        eligible_for_application: false,
        candidate_digest: String::new(),
    };
    candidate.candidate_digest = learned_reward_candidate_digest_v0(&candidate);
    Ok(candidate)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExternalAdmissionCompatibilityV0 {
    PermittedByExistingContracts,
    PermittedWithExternalAdmissionRegistration,
    ForbiddenByMomentumContract,
    ForbiddenByRiskContract,
    ConflictingContracts,
    TechnicalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProspectiveExternalSourceClassV0 {
    ApprovedCredentialFreeProviderExport,
    VerifiedIndependentCanonicalExport,
    UnverifiedOwnerSuppliedRow,
    HistoricalOrConsumedEvidence,
    SyntheticFixture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveRowAdmissionStatusV0 {
    Admitted,
    AwaitingQualifiedExternalRow,
    ContractIncompatible,
    BeforeOrAtCutoff,
    NotFinalized,
    InvalidCanonicalRow,
    DuplicateRow,
    LaterRowAlreadyPresent,
    HistoricalEvidenceReuse,
    UnverifiedSource,
    CredentialOrUnsafeContent,
    LabelLeakageDetected,
    RegistrationMismatch,
    TechnicalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveOperationalOutcomeV0 {
    ShadowPredictionSealed,
    ShadowAbstentionOutOfSupport,
    ShadowAbstentionSupportUnavailable,
    ShadowAbstentionTechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveExternalRowCapsuleV0 {
    pub capsule_version: String,
    pub provider_id: String,
    pub market: String,
    pub symbol: String,
    pub cadence: String,
    pub row: CanonicalHistoricalRowIdentityV1,
    pub source_export_digest: String,
    pub source_class: ProspectiveExternalSourceClassV0,
    pub finalized: bool,
    pub read_only: bool,
    pub sanitized: bool,
    pub credential_free: bool,
    pub acquired_without_model_output_access: bool,
    pub acquired_without_label_access: bool,
    pub candidate_row_count: usize,
    pub contains_unexplained_later_rows: bool,
    pub used_in_consumed_evidence: bool,
    pub contains_label_or_outcome: bool,
    pub model_configuration_digest: String,
    pub capsule_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveExternalAdmissionRegistrationV0 {
    pub registration_version: String,
    pub momentum_challenge_digest: String,
    pub risk_tournament_digest: String,
    pub momentum_cutoff_timestamp: u64,
    pub risk_cutoff_timestamp: u64,
    pub maximum_consumed_evidence_timestamp: u64,
    pub canonical_provider_id: String,
    pub market: String,
    pub symbol: String,
    pub canonical_series_id: String,
    pub cadence: String,
    pub accepted_source_classes: Vec<ProspectiveExternalSourceClassV0>,
    pub frozen_model_configuration_digest: String,
    pub pre_label_isolation_required: bool,
    pub shared_raw_evidence_only: bool,
    pub zero_network_required: bool,
    pub zero_reward_required: bool,
    pub zero_authority_required: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveExternalAdmissionContextV0 {
    pub existing_row_timestamps: BTreeSet<u64>,
    pub existing_canonical_row_digests: BTreeSet<String>,
    pub latest_admitted_timestamp: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedProspectiveRawEvidenceV0 {
    pub reference_version: String,
    pub admission_registration_digest: String,
    pub external_capsule_digest: String,
    pub canonical_row_digest: String,
    pub timestamp: u64,
    pub provider_id: String,
    pub symbol: String,
    pub momentum_cutoff_verified: bool,
    pub risk_cutoff_verified: bool,
    pub eligible_for_momentum_validation: bool,
    pub eligible_for_risk_validation: bool,
    pub label_accessed: bool,
    pub reference_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveExternalChallengeValidationV0 {
    pub validation_version: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub challenge_digest: String,
    pub shared_raw_evidence_digest: String,
    pub independently_valid: bool,
    pub label_accessed: bool,
    pub validation_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedProspectiveEventV0 {
    pub event_version: String,
    pub event_id: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub challenge_digest: String,
    pub shared_raw_evidence_digest: String,
    pub frozen_model_artifact_digests: Vec<String>,
    pub input_digest: String,
    pub support_status_digest: String,
    pub operational_outcome: ProspectiveOperationalOutcomeV0,
    pub abstention_reason: Option<String>,
    pub prediction_timestamp: u64,
    pub maturity_timestamp: u64,
    pub horizon_digest: String,
    pub probability_bits_sealed: bool,
    pub label_accessed: bool,
    pub event_digest: String,
}

fn external_admission_registration_digest_v0(
    registration: &ProspectiveExternalAdmissionRegistrationV0,
) -> String {
    stable_hash_string(&format!(
        "{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}",
        registration.registration_version,
        registration.momentum_challenge_digest,
        registration.risk_tournament_digest,
        registration.momentum_cutoff_timestamp,
        registration.risk_cutoff_timestamp,
        registration.maximum_consumed_evidence_timestamp,
        registration.canonical_provider_id,
        registration.market,
        registration.symbol,
        registration.canonical_series_id,
        registration.cadence,
        registration.accepted_source_classes,
        registration.frozen_model_configuration_digest,
        registration.pre_label_isolation_required,
        registration.shared_raw_evidence_only,
        registration.zero_network_required,
        registration.zero_reward_required,
        registration.zero_authority_required,
    ))
}

fn external_row_capsule_digest_v0(capsule: &ProspectiveExternalRowCapsuleV0) -> String {
    stable_hash_string(&format!(
        "{:?}:{}:{}:{}:{}:{:?}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        capsule.capsule_version,
        capsule.provider_id,
        capsule.market,
        capsule.symbol,
        capsule.cadence,
        capsule.row,
        capsule.source_export_digest,
        capsule.source_class,
        capsule.finalized,
        capsule.read_only,
        capsule.sanitized,
        capsule.credential_free,
        capsule.acquired_without_model_output_access,
        capsule.acquired_without_label_access,
        capsule.candidate_row_count,
        capsule.contains_unexplained_later_rows,
        capsule.used_in_consumed_evidence,
        capsule.contains_label_or_outcome,
        capsule.model_configuration_digest,
    ))
}

/// Finalizes the opaque capsule digest without opening labels or interpreting
/// the row. Admission still performs all contract and canonical-row checks.
pub fn seal_prospective_external_row_capsule_v0(
    mut capsule: ProspectiveExternalRowCapsuleV0,
) -> ProspectiveExternalRowCapsuleV0 {
    capsule.capsule_digest = external_row_capsule_digest_v0(&capsule);
    capsule
}

fn shared_raw_evidence_digest_v0(reference: &SharedProspectiveRawEvidenceV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        reference.reference_version,
        reference.admission_registration_digest,
        reference.external_capsule_digest,
        reference.canonical_row_digest,
        reference.timestamp,
        reference.provider_id,
        reference.symbol,
        reference.momentum_cutoff_verified,
        reference.risk_cutoff_verified,
        reference.eligible_for_momentum_validation,
        reference.eligible_for_risk_validation,
        reference.label_accessed,
    ))
}

fn external_challenge_validation_digest_v0(
    validation: &ProspectiveExternalChallengeValidationV0,
) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{}:{}:{}:{}",
        validation.validation_version,
        validation.agent_id,
        validation.objective,
        validation.challenge_digest,
        validation.shared_raw_evidence_digest,
        validation.independently_valid,
        validation.label_accessed,
    ))
}

fn learned_prospective_event_digest_v0(event: &LearnedProspectiveEventV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{:?}:{}:{}:{:?}:{}:{}:{:?}:{:?}:{}:{}:{}:{}:{}",
        event.event_version,
        event.event_id,
        event.agent_id,
        event.objective,
        event.challenge_digest,
        event.shared_raw_evidence_digest,
        event.frozen_model_artifact_digests,
        event.input_digest,
        event.support_status_digest,
        event.operational_outcome,
        event.abstention_reason,
        event.prediction_timestamp,
        event.maturity_timestamp,
        event.horizon_digest,
        event.probability_bits_sealed,
        event.label_accessed,
    ))
}

fn prospective_external_model_configuration_digest_v0(
    momentum: &ProspectiveChallengeLocalStateV0,
    risk: &CycleRiskProspectiveTournamentCapsuleV0,
) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        momentum.capsule.candidate.artifact_digest,
        momentum.capsule.feature_policy_digest,
        momentum.capsule.label_policy_digest,
        momentum.capsule.support_policy_digest,
        risk.historical_champion.artifact_digest,
        risk.experimental_challenger.artifact_digest,
        risk.minimum_benchmark.artifact_digest,
        risk.feature_policy_digest,
        risk.label_policy_digest,
        risk.support_policy_digest,
        risk.collapse_policy_digest,
        risk.error_audit_policy_digest,
    ))
}

pub fn audit_external_admission_compatibility_v0(
    momentum: &ProspectiveChallengeLocalStateV0,
    risk: &CycleRiskProspectiveTournamentCapsuleV0,
) -> ExternalAdmissionCompatibilityV0 {
    if validate_prospective_challenge_local_state_v0(momentum).is_err() {
        return ExternalAdmissionCompatibilityV0::ForbiddenByMomentumContract;
    }
    if validate_cycle_risk_prospective_capsule_v0(risk).is_err() {
        return ExternalAdmissionCompatibilityV0::ForbiddenByRiskContract;
    }
    let canonical_momentum_series = momentum
        .capsule
        .series_id
        .rsplit_once(':')
        .map(|(_, series)| series)
        .unwrap_or(&momentum.capsule.series_id);
    let canonical_risk_series = risk
        .series_id
        .rsplit_once(':')
        .map(|(_, series)| series)
        .unwrap_or(&risk.series_id);
    if canonical_momentum_series != canonical_risk_series
        || momentum.capsule.prospective_cutoff_exclusive_timestamp_ms == 0
        || risk.cutoff_exclusive_timestamp_ms == 0
    {
        return ExternalAdmissionCompatibilityV0::ConflictingContracts;
    }
    if !momentum.capsule.evidence_policy.finalized_daily_rows_only
        || momentum.capsule.evidence_policy.maximum_requests != 1
        || momentum.capsule.evidence_policy.maximum_concurrency != 1
        || momentum.capsule.evidence_policy.retry_count != 0
        || !momentum
            .capsule
            .prediction_policy
            .hide_labels_before_opening
        || !momentum
            .capsule
            .prediction_policy
            .hide_probabilities_before_opening
    {
        return ExternalAdmissionCompatibilityV0::ForbiddenByMomentumContract;
    }
    if !risk.evidence_policy.finalized_daily_rows_only
        || !risk.evidence_policy.accepts_shared_acquisition_epochs
        || risk.evidence_policy.maximum_requests != 1
        || risk.evidence_policy.maximum_concurrency != 1
        || risk.evidence_policy.maximum_retries != 0
    {
        return ExternalAdmissionCompatibilityV0::ForbiddenByRiskContract;
    }
    ExternalAdmissionCompatibilityV0::PermittedWithExternalAdmissionRegistration
}

pub fn pre_register_prospective_external_row_admission_v0(
    momentum: &ProspectiveChallengeLocalStateV0,
    risk: &CycleRiskProspectiveTournamentCapsuleV0,
    maximum_consumed_evidence_timestamp: u64,
) -> Result<ProspectiveExternalAdmissionRegistrationV0, String> {
    if audit_external_admission_compatibility_v0(momentum, risk)
        != ExternalAdmissionCompatibilityV0::PermittedWithExternalAdmissionRegistration
    {
        return Err("prospective_external_admission_contract_incompatible".into());
    }
    let canonical_series_id = momentum.capsule.series_id.clone();
    let (market, symbol) = canonical_series_id
        .split_once(':')
        .map(|(market, symbol)| (market.to_string(), symbol.to_string()))
        .or_else(|| {
            canonical_series_id.split_once('-').map(|(left, right)| {
                if left.eq_ignore_ascii_case("BTC") {
                    (right.to_string(), left.to_string())
                } else {
                    (left.to_string(), right.to_string())
                }
            })
        })
        .unwrap_or_else(|| ("spot".into(), canonical_series_id.clone()));
    let mut registration = ProspectiveExternalAdmissionRegistrationV0 {
        registration_version: "prospective-external-row-admission-registration-v0".into(),
        momentum_challenge_digest: momentum.capsule.capsule_digest.clone(),
        risk_tournament_digest: risk.capsule_digest.clone(),
        momentum_cutoff_timestamp: momentum.capsule.prospective_cutoff_exclusive_timestamp_ms,
        risk_cutoff_timestamp: risk.cutoff_exclusive_timestamp_ms,
        maximum_consumed_evidence_timestamp,
        canonical_provider_id: "approved-credential-free-external-export-v0".into(),
        market,
        symbol,
        canonical_series_id,
        cadence: "1d".into(),
        accepted_source_classes: vec![
            ProspectiveExternalSourceClassV0::ApprovedCredentialFreeProviderExport,
            ProspectiveExternalSourceClassV0::VerifiedIndependentCanonicalExport,
        ],
        frozen_model_configuration_digest: prospective_external_model_configuration_digest_v0(
            momentum, risk,
        ),
        pre_label_isolation_required: true,
        shared_raw_evidence_only: true,
        zero_network_required: true,
        zero_reward_required: true,
        zero_authority_required: true,
        registration_digest: String::new(),
    };
    registration.registration_digest = external_admission_registration_digest_v0(&registration);
    validate_prospective_external_admission_registration_v0(&registration, momentum, risk)?;
    Ok(registration)
}

pub fn validate_prospective_external_admission_registration_v0(
    registration: &ProspectiveExternalAdmissionRegistrationV0,
    momentum: &ProspectiveChallengeLocalStateV0,
    risk: &CycleRiskProspectiveTournamentCapsuleV0,
) -> Result<(), String> {
    let compatibility = audit_external_admission_compatibility_v0(momentum, risk);
    let expected_model_digest = prospective_external_model_configuration_digest_v0(momentum, risk);
    let expected_sources = vec![
        ProspectiveExternalSourceClassV0::ApprovedCredentialFreeProviderExport,
        ProspectiveExternalSourceClassV0::VerifiedIndependentCanonicalExport,
    ];
    if compatibility != ExternalAdmissionCompatibilityV0::PermittedWithExternalAdmissionRegistration
        || registration.registration_version != "prospective-external-row-admission-registration-v0"
        || registration.momentum_challenge_digest != momentum.capsule.capsule_digest
        || registration.risk_tournament_digest != risk.capsule_digest
        || registration.momentum_cutoff_timestamp
            != momentum.capsule.prospective_cutoff_exclusive_timestamp_ms
        || registration.risk_cutoff_timestamp != risk.cutoff_exclusive_timestamp_ms
        || registration.maximum_consumed_evidence_timestamp
            < registration
                .momentum_cutoff_timestamp
                .max(registration.risk_cutoff_timestamp)
        || registration.canonical_provider_id.is_empty()
        || registration.market.is_empty()
        || registration.symbol.is_empty()
        || registration.canonical_series_id != momentum.capsule.series_id
        || registration.cadence != "1d"
        || registration.accepted_source_classes != expected_sources
        || registration.frozen_model_configuration_digest != expected_model_digest
        || !registration.pre_label_isolation_required
        || !registration.shared_raw_evidence_only
        || !registration.zero_network_required
        || !registration.zero_reward_required
        || !registration.zero_authority_required
        || registration.registration_digest
            != external_admission_registration_digest_v0(registration)
    {
        Err("prospective_external_admission_registration_invalid".into())
    } else {
        Ok(())
    }
}

pub fn write_prospective_external_admission_registration_v0(
    path: &Path,
    registration: &ProspectiveExternalAdmissionRegistrationV0,
    momentum: &ProspectiveChallengeLocalStateV0,
    risk: &CycleRiskProspectiveTournamentCapsuleV0,
) -> Result<(), String> {
    validate_prospective_external_admission_registration_v0(registration, momentum, risk)?;
    let parent = path
        .parent()
        .ok_or("external_admission_registration_storage_unavailable")?;
    fs::create_dir_all(parent)
        .map_err(|_| "external_admission_registration_storage_unavailable")?;
    let encoded = serde_json::to_vec(registration)
        .map_err(|_| "external_admission_registration_serialization_failed")?;
    let temp = path.with_extension("tmp");
    fs::write(&temp, encoded).map_err(|_| "external_admission_registration_storage_failed")?;
    fs::rename(temp, path).map_err(|_| "external_admission_registration_storage_failed".to_string())
}

pub fn read_prospective_external_admission_registration_v0(
    path: &Path,
) -> Result<ProspectiveExternalAdmissionRegistrationV0, String> {
    serde_json::from_slice(
        &fs::read(path).map_err(|_| "external_admission_registration_unavailable")?,
    )
    .map_err(|_| "external_admission_registration_invalid".into())
}

pub fn read_prospective_external_row_capsule_v0(
    path: &Path,
) -> Result<ProspectiveExternalRowCapsuleV0, String> {
    serde_json::from_slice(&fs::read(path).map_err(|_| "external_row_capsule_unavailable")?)
        .map_err(|_| "external_row_capsule_invalid".into())
}

fn canonical_external_row_is_valid_v0(row: &CanonicalHistoricalRowIdentityV1) -> bool {
    let open = f64::from_bits(row.open_bits);
    let high = f64::from_bits(row.high_bits);
    let low = f64::from_bits(row.low_bits);
    let close = f64::from_bits(row.close_bits);
    let volume = f64::from_bits(row.volume_bits);
    row.timestamp_ms != 0
        && [open, high, low, close, volume]
            .iter()
            .all(|value| value.is_finite())
        && row
            .trade_value_bits
            .is_none_or(|bits| f64::from_bits(bits).is_finite())
        && open > 0.0
        && high > 0.0
        && low > 0.0
        && close > 0.0
        && low <= open.min(close)
        && high >= open.max(close)
        && high >= low
        && volume >= 0.0
        && row.row_digest_v1 == canonical_semantic_digest_v1(row)
}

pub fn prospective_external_row_admission_status_v0(
    registration: &ProspectiveExternalAdmissionRegistrationV0,
    momentum: &ProspectiveChallengeLocalStateV0,
    risk: &CycleRiskProspectiveTournamentCapsuleV0,
    capsule: &ProspectiveExternalRowCapsuleV0,
    context: &ProspectiveExternalAdmissionContextV0,
) -> ProspectiveRowAdmissionStatusV0 {
    if validate_prospective_external_admission_registration_v0(registration, momentum, risk)
        .is_err()
    {
        return ProspectiveRowAdmissionStatusV0::RegistrationMismatch;
    }
    if capsule.capsule_version != "prospective-external-row-capsule-v0"
        || capsule.capsule_digest != external_row_capsule_digest_v0(capsule)
    {
        return ProspectiveRowAdmissionStatusV0::TechnicalFailure;
    }
    if !registration
        .accepted_source_classes
        .contains(&capsule.source_class)
    {
        return if capsule.source_class
            == ProspectiveExternalSourceClassV0::HistoricalOrConsumedEvidence
        {
            ProspectiveRowAdmissionStatusV0::HistoricalEvidenceReuse
        } else {
            ProspectiveRowAdmissionStatusV0::UnverifiedSource
        };
    }
    if !capsule.read_only || !capsule.sanitized || !capsule.credential_free {
        return ProspectiveRowAdmissionStatusV0::CredentialOrUnsafeContent;
    }
    if !capsule.finalized {
        return ProspectiveRowAdmissionStatusV0::NotFinalized;
    }
    if capsule.candidate_row_count != 1 || capsule.contains_unexplained_later_rows {
        return ProspectiveRowAdmissionStatusV0::TechnicalFailure;
    }
    if capsule.used_in_consumed_evidence {
        return ProspectiveRowAdmissionStatusV0::HistoricalEvidenceReuse;
    }
    if capsule.contains_label_or_outcome
        || !capsule.acquired_without_model_output_access
        || !capsule.acquired_without_label_access
    {
        return ProspectiveRowAdmissionStatusV0::LabelLeakageDetected;
    }
    if capsule.provider_id != registration.canonical_provider_id
        || capsule.market != registration.market
        || capsule.symbol != registration.symbol
        || capsule.cadence != registration.cadence
        || capsule.row.provider_id != capsule.provider_id
        || capsule.row.series_id != registration.canonical_series_id
        || capsule.model_configuration_digest != registration.frozen_model_configuration_digest
        || !canonical_external_row_is_valid_v0(&capsule.row)
    {
        return ProspectiveRowAdmissionStatusV0::InvalidCanonicalRow;
    }
    let cutoff = registration
        .momentum_cutoff_timestamp
        .max(registration.risk_cutoff_timestamp)
        .max(registration.maximum_consumed_evidence_timestamp);
    if capsule.row.timestamp_ms <= cutoff {
        return ProspectiveRowAdmissionStatusV0::BeforeOrAtCutoff;
    }
    if context
        .existing_row_timestamps
        .contains(&capsule.row.timestamp_ms)
        || context
            .existing_canonical_row_digests
            .contains(&capsule.row.row_digest_v1)
    {
        return ProspectiveRowAdmissionStatusV0::DuplicateRow;
    }
    if context
        .latest_admitted_timestamp
        .is_some_and(|latest| latest > capsule.row.timestamp_ms)
    {
        return ProspectiveRowAdmissionStatusV0::LaterRowAlreadyPresent;
    }
    ProspectiveRowAdmissionStatusV0::Admitted
}

pub fn build_shared_prospective_raw_evidence_v0(
    registration: &ProspectiveExternalAdmissionRegistrationV0,
    capsule: &ProspectiveExternalRowCapsuleV0,
    admission_status: ProspectiveRowAdmissionStatusV0,
) -> Result<SharedProspectiveRawEvidenceV0, String> {
    if admission_status != ProspectiveRowAdmissionStatusV0::Admitted
        || capsule.row.timestamp_ms <= registration.momentum_cutoff_timestamp
        || capsule.row.timestamp_ms <= registration.risk_cutoff_timestamp
        || capsule.row.timestamp_ms <= registration.maximum_consumed_evidence_timestamp
        || capsule.capsule_digest != external_row_capsule_digest_v0(capsule)
    {
        return Err("shared_prospective_raw_evidence_not_admissible".into());
    }
    let mut reference = SharedProspectiveRawEvidenceV0 {
        reference_version: "shared-prospective-raw-evidence-v0".into(),
        admission_registration_digest: registration.registration_digest.clone(),
        external_capsule_digest: capsule.capsule_digest.clone(),
        canonical_row_digest: capsule.row.row_digest_v1.clone(),
        timestamp: capsule.row.timestamp_ms,
        provider_id: capsule.provider_id.clone(),
        symbol: capsule.symbol.clone(),
        momentum_cutoff_verified: true,
        risk_cutoff_verified: true,
        eligible_for_momentum_validation: true,
        eligible_for_risk_validation: true,
        label_accessed: false,
        reference_digest: String::new(),
    };
    reference.reference_digest = shared_raw_evidence_digest_v0(&reference);
    Ok(reference)
}

fn new_external_challenge_validation_v0(
    agent_id: &str,
    objective: LearnedAgentObjectiveV0,
    challenge_digest: String,
    shared: &SharedProspectiveRawEvidenceV0,
    independently_valid: bool,
) -> ProspectiveExternalChallengeValidationV0 {
    let mut validation = ProspectiveExternalChallengeValidationV0 {
        validation_version: "prospective-external-challenge-validation-v0".into(),
        agent_id: agent_id.into(),
        objective,
        challenge_digest,
        shared_raw_evidence_digest: shared.reference_digest.clone(),
        independently_valid,
        label_accessed: false,
        validation_digest: String::new(),
    };
    validation.validation_digest = external_challenge_validation_digest_v0(&validation);
    validation
}

pub fn validate_momentum_shared_prospective_reference_v0(
    registration: &ProspectiveExternalAdmissionRegistrationV0,
    momentum: &ProspectiveChallengeLocalStateV0,
    shared: &SharedProspectiveRawEvidenceV0,
) -> ProspectiveExternalChallengeValidationV0 {
    let valid = validate_prospective_challenge_local_state_v0(momentum).is_ok()
        && registration.momentum_challenge_digest == momentum.capsule.capsule_digest
        && momentum.capsule.status == ProspectiveChallengeStatusV0::Sealed
        && momentum.capsule.candidate.shadow_only
        && momentum.capsule.comparators.len() == 2
        && momentum.capsule.prediction_horizon > 0
        && shared.admission_registration_digest == registration.registration_digest
        && shared.timestamp > momentum.capsule.prospective_cutoff_exclusive_timestamp_ms
        && shared.momentum_cutoff_verified
        && shared.eligible_for_momentum_validation
        && !shared.label_accessed;
    new_external_challenge_validation_v0(
        MOMENTUM_AGENT_ID_V0,
        LearnedAgentObjectiveV0::DirectionalMomentum,
        momentum.capsule.capsule_digest.clone(),
        shared,
        valid,
    )
}

pub fn validate_risk_shared_prospective_reference_v0(
    registration: &ProspectiveExternalAdmissionRegistrationV0,
    risk: &CycleRiskProspectiveLocalStateV0,
    shared: &SharedProspectiveRawEvidenceV0,
) -> ProspectiveExternalChallengeValidationV0 {
    let valid = validate_cycle_risk_prospective_local_state_v0(risk).is_ok()
        && registration.risk_tournament_digest == risk.capsule.capsule_digest
        && risk.capsule.status == CycleRiskProspectiveChallengeStatusV0::Sealed
        && risk.capsule.historical_champion.artifact_digest
            != risk.capsule.experimental_challenger.artifact_digest
        && risk.capsule.experimental_challenger.artifact_digest
            != risk.capsule.minimum_benchmark.artifact_digest
        && risk.capsule.prediction_horizon > 0
        && shared.admission_registration_digest == registration.registration_digest
        && shared.timestamp > risk.capsule.cutoff_exclusive_timestamp_ms
        && shared.risk_cutoff_verified
        && shared.eligible_for_risk_validation
        && !shared.label_accessed;
    new_external_challenge_validation_v0(
        CYCLE_RISK_SHADOW_AGENT_ID_V0,
        LearnedAgentObjectiveV0::DownsideRisk,
        risk.capsule.capsule_digest.clone(),
        shared,
        valid,
    )
}

fn external_event_artifacts_v0(
    objective: LearnedAgentObjectiveV0,
    momentum: &ProspectiveChallengeLocalStateV0,
    risk: &CycleRiskProspectiveLocalStateV0,
) -> (String, Vec<String>, usize) {
    match objective {
        LearnedAgentObjectiveV0::DirectionalMomentum => (
            momentum.capsule.capsule_digest.clone(),
            std::iter::once(momentum.capsule.candidate.artifact_digest.clone())
                .chain(
                    momentum
                        .capsule
                        .comparators
                        .iter()
                        .map(|value| value.artifact_digest.clone()),
                )
                .collect(),
            momentum.capsule.prediction_horizon,
        ),
        LearnedAgentObjectiveV0::DownsideRisk => (
            risk.capsule.capsule_digest.clone(),
            vec![
                risk.capsule.historical_champion.artifact_digest.clone(),
                risk.capsule.experimental_challenger.artifact_digest.clone(),
                risk.capsule.minimum_benchmark.artifact_digest.clone(),
            ],
            risk.capsule.prediction_horizon,
        ),
    }
}

pub fn seal_external_prospective_event_v0(
    validation: &ProspectiveExternalChallengeValidationV0,
    shared: &SharedProspectiveRawEvidenceV0,
    momentum: &ProspectiveChallengeLocalStateV0,
    risk: &CycleRiskProspectiveLocalStateV0,
    operational_outcome: ProspectiveOperationalOutcomeV0,
    abstention_reason: Option<String>,
) -> Result<LearnedProspectiveEventV0, String> {
    if !validation.independently_valid
        || validation.label_accessed
        || validation.shared_raw_evidence_digest != shared.reference_digest
        || shared.label_accessed
        || (operational_outcome == ProspectiveOperationalOutcomeV0::ShadowPredictionSealed
            && abstention_reason.is_some())
        || (operational_outcome != ProspectiveOperationalOutcomeV0::ShadowPredictionSealed
            && abstention_reason.as_deref().is_none_or(str::is_empty))
    {
        return Err("external_prospective_event_sealing_invalid".into());
    }
    let expected_agent = match validation.objective {
        LearnedAgentObjectiveV0::DirectionalMomentum => MOMENTUM_AGENT_ID_V0,
        LearnedAgentObjectiveV0::DownsideRisk => CYCLE_RISK_SHADOW_AGENT_ID_V0,
    };
    let (challenge_digest, mut artifacts, horizon_rows) =
        external_event_artifacts_v0(validation.objective, momentum, risk);
    if validation.agent_id != expected_agent
        || validation.challenge_digest != challenge_digest
        || horizon_rows == 0
    {
        return Err("external_prospective_event_contract_mismatch".into());
    }
    artifacts.sort();
    let horizon_digest = stable_hash_string(&format!(
        "external-prospective-horizon-v0:{:?}:{}",
        validation.objective, horizon_rows
    ));
    let event_id = format!(
        "external-prospective-event-{}",
        stable_hash_string(&format!(
            "{}:{}:{:?}:{}",
            validation.validation_digest,
            shared.reference_digest,
            validation.objective,
            horizon_digest
        ))
    );
    let mut event = LearnedProspectiveEventV0 {
        event_version: "learned-prospective-event-v0".into(),
        event_id,
        agent_id: expected_agent.into(),
        objective: validation.objective,
        challenge_digest,
        shared_raw_evidence_digest: shared.reference_digest.clone(),
        frozen_model_artifact_digests: artifacts,
        input_digest: shared.reference_digest.clone(),
        support_status_digest: stable_hash_string(&format!(
            "external-prospective-support-v0:{:?}:{:?}",
            validation.objective, operational_outcome
        )),
        operational_outcome,
        abstention_reason,
        prediction_timestamp: shared.timestamp,
        maturity_timestamp: shared
            .timestamp
            .saturating_add(horizon_rows as u64 * 86_400_000),
        horizon_digest,
        probability_bits_sealed: true,
        label_accessed: false,
        event_digest: String::new(),
    };
    if event.maturity_timestamp <= event.prediction_timestamp {
        return Err("external_prospective_event_maturity_invalid".into());
    }
    event.event_digest = learned_prospective_event_digest_v0(&event);
    Ok(event)
}

pub fn external_admission_reward_eligibility_status_v0(
    sealed_event_count: usize,
) -> LearnedRewardEligibilityStatusV0 {
    if sealed_event_count == 0 {
        LearnedRewardEligibilityStatusV0::IneligibleNoProspectiveOutcomes
    } else {
        LearnedRewardEligibilityStatusV0::IneligibleAwaitingMaturity
    }
}

fn momentum_external_event_already_persisted_v0(
    state: &ProspectiveChallengeLocalStateV0,
    shared: &SharedProspectiveRawEvidenceV0,
    event: &LearnedProspectiveEventV0,
) -> bool {
    state.vault.finalized_rows.len() == 1
        && state.vault.finalized_rows[0].timestamp_ms == shared.timestamp
        && state.vault.finalized_rows[0].canonical_row_digest == shared.canonical_row_digest
        && state.journal.events.len() == 1
        && state.journal.events[0].event_id == event.event_id
        && state.journal.events[0].input_evidence_digest == shared.reference_digest
        && state.journal.events[0].prediction_timestamp_ms == event.prediction_timestamp
}

fn prospective_outcome_for_local_journal_v0(
    outcome: ProspectiveOperationalOutcomeV0,
) -> Result<ProspectiveShadowOutcomeV0, String> {
    match outcome {
        ProspectiveOperationalOutcomeV0::ShadowPredictionSealed => {
            Err("external_prospective_prediction_bits_not_available".into())
        }
        ProspectiveOperationalOutcomeV0::ShadowAbstentionOutOfSupport => {
            Ok(ProspectiveShadowOutcomeV0::ShadowAbstainOutOfSupport)
        }
        ProspectiveOperationalOutcomeV0::ShadowAbstentionSupportUnavailable => {
            Ok(ProspectiveShadowOutcomeV0::ShadowAbstainSupportUnavailable)
        }
        ProspectiveOperationalOutcomeV0::ShadowAbstentionTechnicalFailure => {
            Ok(ProspectiveShadowOutcomeV0::ShadowAbstainNumericalFailure)
        }
    }
}

fn validate_persistable_external_event_v0(
    event: &LearnedProspectiveEventV0,
    shared: &SharedProspectiveRawEvidenceV0,
    expected_objective: LearnedAgentObjectiveV0,
    expected_challenge_digest: &str,
) -> Result<(), String> {
    if event.objective != expected_objective
        || event.challenge_digest != expected_challenge_digest
        || event.shared_raw_evidence_digest != shared.reference_digest
        || event.input_digest != shared.reference_digest
        || event.prediction_timestamp != shared.timestamp
        || event.maturity_timestamp <= event.prediction_timestamp
        || !event.probability_bits_sealed
        || event.label_accessed
        || event.event_digest != learned_prospective_event_digest_v0(event)
        || event.operational_outcome == ProspectiveOperationalOutcomeV0::ShadowPredictionSealed
        || event.abstention_reason.as_deref().is_none_or(str::is_empty)
    {
        Err("external_prospective_event_not_persistable".into())
    } else {
        Ok(())
    }
}

/// Atomically fans one already-admitted raw row out into the independent local
/// journals. Only redacted evidence and event digests are persisted; each
/// agent must be validated and sealed independently before it can participate.
pub fn append_external_admission_to_local_stores_v0(
    momentum: &mut ProspectiveChallengeLocalStateV0,
    risk: &mut CycleRiskProspectiveLocalStateV0,
    shared: &SharedProspectiveRawEvidenceV0,
    momentum_event: Option<&LearnedProspectiveEventV0>,
    risk_event: Option<&LearnedProspectiveEventV0>,
) -> Result<(), String> {
    if shared.label_accessed
        || shared.reference_digest != shared_raw_evidence_digest_v0(shared)
        || (momentum_event.is_none() && risk_event.is_none())
    {
        return Err("external_prospective_local_append_invalid".into());
    }
    let mut next_momentum = momentum.clone();
    let mut next_risk = risk.clone();
    if let Some(event) = momentum_event {
        validate_persistable_external_event_v0(
            event,
            shared,
            LearnedAgentObjectiveV0::DirectionalMomentum,
            &next_momentum.capsule.capsule_digest,
        )?;
        if !momentum_external_event_already_persisted_v0(&next_momentum, shared, event) {
            append_prospective_vault_row_v0(
                &mut next_momentum,
                ProspectiveEvidenceRowRefV0 {
                    timestamp_ms: shared.timestamp,
                    canonical_row_digest: shared.canonical_row_digest.clone(),
                    finalized: true,
                },
            )
            .map_err(|_| "external_prospective_momentum_vault_append_failed")?;
            let local_event = ProspectivePredictionEventV0 {
                challenge_id: next_momentum.capsule.challenge_id.clone(),
                event_id: event.event_id.clone(),
                prediction_timestamp_ms: event.prediction_timestamp,
                required_label_maturity_timestamp_ms: event.maturity_timestamp,
                input_evidence_digest: shared.reference_digest.clone(),
                candidate_artifact_digest: next_momentum.capsule.candidate.artifact_digest.clone(),
                comparator_artifact_digests: next_momentum
                    .capsule
                    .comparators
                    .iter()
                    .map(|value| value.artifact_digest.clone())
                    .collect(),
                support_applicability: "unavailable".into(),
                support_decision: "abstain".into(),
                candidate_prediction: None,
                comparator_predictions: vec![],
                operational_outcome: prospective_outcome_for_local_journal_v0(
                    event.operational_outcome,
                )?,
                label_status: ProspectiveLabelStatusV0::AwaitingFutureRows,
                event_digest: String::new(),
            };
            append_prospective_prediction_event_v0(&mut next_momentum, local_event)
                .map_err(|_| "external_prospective_momentum_event_append_failed")?;
        }
    }
    if let Some(event) = risk_event {
        validate_persistable_external_event_v0(
            event,
            shared,
            LearnedAgentObjectiveV0::DownsideRisk,
            &next_risk.capsule.capsule_digest,
        )?;
        append_cycle_risk_external_row_and_event_v0(
            &mut next_risk,
            shared.timestamp,
            shared.canonical_row_digest.clone(),
            event.event_digest.clone(),
        )
        .map_err(|_| "external_prospective_risk_journal_append_failed")?;
    }
    validate_prospective_challenge_local_state_v0(&next_momentum)
        .map_err(|_| "external_prospective_momentum_post_append_invalid")?;
    validate_cycle_risk_prospective_local_state_v0(&next_risk)
        .map_err(|_| "external_prospective_risk_post_append_invalid")?;
    *momentum = next_momentum;
    *risk = next_risk;
    Ok(())
}

const PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0: u64 = 86_400_000;
pub const MAXIMUM_PROSPECTIVE_OUTCOME_RESPONSE_ROWS_V0: usize = 31;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveOpeningReadinessV0 {
    AwaitingTimeMaturity,
    TimeMatureOutcomeRowsMissing,
    OutcomeRowsPresentButUnverified,
    ReadyForExplicitOpening,
    AlreadyOpened,
    IntegrityInvalid,
    ChallengeInvalid,
    TechnicalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveOutcomeRequestReadinessV0 {
    AwaitingMomentumTimeMaturity,
    AwaitingRiskTimeMaturity,
    AwaitingBothTimeMaturities,
    ReadyForExplicitRequest,
    RequestAlreadyAttempted,
    OutcomeEvidenceAlreadyPresent,
    RegistrationInvalid,
    EventIntegrityInvalid,
    TechnicalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProspectiveOutcomeEvidenceStatusV0 {
    NoOutcomeRows,
    PartialOutcomeRows,
    CompleteUnverified,
    CompleteVerified,
    DuplicateRows,
    MissingRequiredTimestamp,
    NonFinalizedRow,
    WrongSeries,
    ChronologyInvalid,
    IntegrityInvalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveEventMaturityPlanV0 {
    pub plan_version: String,
    pub event_id: String,
    pub event_digest: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub prediction_timestamp: u64,
    pub maturity_timestamp: u64,
    pub horizon_digest: String,
    pub required_outcome_start_timestamp: u64,
    pub required_outcome_end_timestamp: u64,
    pub required_finalized_row_count: usize,
    pub label_policy_digest: String,
    pub source_policy_digest: String,
    pub plan_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveOneTimeOpeningRegistrationV0 {
    pub registration_version: String,
    pub momentum_event_digest: String,
    pub risk_event_digest: String,
    pub maturity_plan_digests: Vec<String>,
    pub shared_raw_evidence_digest: String,
    pub outcome_source_policy_digest: String,
    pub finalization_policy_digest: String,
    pub label_policy_digests: Vec<String>,
    pub metric_policy_digests: Vec<String>,
    pub maximum_future_requests: usize,
    pub maximum_concurrency: usize,
    pub maximum_retries: usize,
    pub maximum_response_rows: usize,
    pub explicit_opening_authorization_required: bool,
    pub one_time_opening_required: bool,
    pub early_opening_forbidden: bool,
    pub duplicate_opening_forbidden: bool,
    pub interim_metrics_forbidden: bool,
    pub network_execution_allowed_this_sprint: bool,
    pub label_access_allowed_this_sprint: bool,
    pub reward_application_allowed: bool,
    pub registration_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProspectiveOpeningAuthorizationV0 {
    pub authorization_version: String,
    pub opening_registration_digest: String,
    pub authorized_event_digests: Vec<String>,
    pub authorized_outcome_evidence_digest: String,
    pub explicit_owner_authorization: bool,
    pub one_time_only: bool,
    pub label_open_count_before: usize,
    pub authorization_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveOutcomeEvidenceRowV0 {
    pub series_id: String,
    pub timestamp: u64,
    pub canonical_row_digest: String,
    pub finalized: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveOutcomeEvidenceAssessmentV0 {
    pub status: ProspectiveOutcomeEvidenceStatusV0,
    pub required_finalized_row_count: usize,
    pub observed_row_count: usize,
    pub evidence_digest: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProspectiveSealedEventAuditV0 {
    pub shared_raw_evidence_digest: String,
    pub momentum_event: LearnedProspectiveEventV0,
    pub risk_event: LearnedProspectiveEventV0,
}

fn prospective_event_maturity_plan_digest_v0(plan: &ProspectiveEventMaturityPlanV0) -> String {
    stable_hash_string(&format!(
        "{}:{}:{}:{}:{:?}:{}:{}:{}:{}:{}:{}:{}:{}",
        plan.plan_version,
        plan.event_id,
        plan.event_digest,
        plan.agent_id,
        plan.objective,
        plan.prediction_timestamp,
        plan.maturity_timestamp,
        plan.horizon_digest,
        plan.required_outcome_start_timestamp,
        plan.required_outcome_end_timestamp,
        plan.required_finalized_row_count,
        plan.label_policy_digest,
        plan.source_policy_digest,
    ))
}

fn prospective_one_time_opening_registration_digest_v0(
    registration: &ProspectiveOneTimeOpeningRegistrationV0,
) -> String {
    stable_hash_string(
        &[
            registration.registration_version.clone(),
            registration.momentum_event_digest.clone(),
            registration.risk_event_digest.clone(),
            registration.maturity_plan_digests.join(","),
            registration.shared_raw_evidence_digest.clone(),
            registration.outcome_source_policy_digest.clone(),
            registration.finalization_policy_digest.clone(),
            registration.label_policy_digests.join(","),
            registration.metric_policy_digests.join(","),
            registration.maximum_future_requests.to_string(),
            registration.maximum_concurrency.to_string(),
            registration.maximum_retries.to_string(),
            registration.maximum_response_rows.to_string(),
            registration
                .explicit_opening_authorization_required
                .to_string(),
            registration.one_time_opening_required.to_string(),
            registration.early_opening_forbidden.to_string(),
            registration.duplicate_opening_forbidden.to_string(),
            registration.interim_metrics_forbidden.to_string(),
            registration
                .network_execution_allowed_this_sprint
                .to_string(),
            registration.label_access_allowed_this_sprint.to_string(),
            registration.reward_application_allowed.to_string(),
        ]
        .join(":"),
    )
}

fn prospective_opening_authorization_digest_v0(
    authorization: &ProspectiveOpeningAuthorizationV0,
) -> String {
    stable_hash_string(&format!(
        "{}:{}:{:?}:{}:{}:{}:{}",
        authorization.authorization_version,
        authorization.opening_registration_digest,
        authorization.authorized_event_digests,
        authorization.authorized_outcome_evidence_digest,
        authorization.explicit_owner_authorization,
        authorization.one_time_only,
        authorization.label_open_count_before,
    ))
}

fn sealed_external_event_horizon_rows_v0(
    event: &LearnedProspectiveEventV0,
    expected_agent_id: &str,
    expected_objective: LearnedAgentObjectiveV0,
) -> Result<usize, String> {
    if event.event_version != "learned-prospective-event-v0"
        || event.agent_id != expected_agent_id
        || event.objective != expected_objective
        || event.event_digest != learned_prospective_event_digest_v0(event)
        || event.event_id.is_empty()
        || event.challenge_digest.is_empty()
        || event.shared_raw_evidence_digest.is_empty()
        || event.frozen_model_artifact_digests.is_empty()
        || event.input_digest != event.shared_raw_evidence_digest
        || event.support_status_digest.is_empty()
        || event.prediction_timestamp == 0
        || event.maturity_timestamp <= event.prediction_timestamp
        || !event.probability_bits_sealed
        || event.label_accessed
    {
        return Err("prospective_sealed_event_integrity_invalid".into());
    }
    let interval = event.maturity_timestamp - event.prediction_timestamp;
    if interval % PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0 != 0 {
        return Err("prospective_sealed_event_horizon_invalid".into());
    }
    let row_count = (interval / PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0) as usize;
    if row_count == 0
        || event.horizon_digest
            != stable_hash_string(&format!(
                "external-prospective-horizon-v0:{expected_objective:?}:{row_count}"
            ))
    {
        return Err("prospective_sealed_event_horizon_invalid".into());
    }
    Ok(row_count)
}

pub fn derive_prospective_event_maturity_plan_v0(
    event: &LearnedProspectiveEventV0,
    sealed_horizon_rows: usize,
    label_policy_digest: &str,
    source_policy_digest: &str,
) -> Result<ProspectiveEventMaturityPlanV0, String> {
    let (expected_agent_id, expected_objective) = match event.objective {
        LearnedAgentObjectiveV0::DirectionalMomentum => (
            MOMENTUM_AGENT_ID_V0,
            LearnedAgentObjectiveV0::DirectionalMomentum,
        ),
        LearnedAgentObjectiveV0::DownsideRisk => (
            CYCLE_RISK_SHADOW_AGENT_ID_V0,
            LearnedAgentObjectiveV0::DownsideRisk,
        ),
    };
    let derived_horizon_rows =
        sealed_external_event_horizon_rows_v0(event, expected_agent_id, expected_objective)?;
    if sealed_horizon_rows == 0
        || sealed_horizon_rows != derived_horizon_rows
        || label_policy_digest.is_empty()
        || source_policy_digest.is_empty()
    {
        return Err("prospective_maturity_plan_contract_invalid".into());
    }
    let required_outcome_start_timestamp = event
        .prediction_timestamp
        .checked_add(PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0)
        .ok_or("prospective_maturity_plan_range_invalid")?;
    let required_outcome_end_timestamp = required_outcome_start_timestamp
        .checked_add(
            (sealed_horizon_rows.saturating_sub(1) as u64)
                .saturating_mul(PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0),
        )
        .ok_or("prospective_maturity_plan_range_invalid")?;
    if required_outcome_end_timestamp != event.maturity_timestamp {
        return Err("prospective_maturity_plan_range_invalid".into());
    }
    let mut plan = ProspectiveEventMaturityPlanV0 {
        plan_version: "prospective-event-maturity-plan-v0".into(),
        event_id: event.event_id.clone(),
        event_digest: event.event_digest.clone(),
        agent_id: event.agent_id.clone(),
        objective: event.objective,
        prediction_timestamp: event.prediction_timestamp,
        maturity_timestamp: event.maturity_timestamp,
        horizon_digest: event.horizon_digest.clone(),
        required_outcome_start_timestamp,
        required_outcome_end_timestamp,
        required_finalized_row_count: sealed_horizon_rows,
        label_policy_digest: label_policy_digest.into(),
        source_policy_digest: source_policy_digest.into(),
        plan_digest: String::new(),
    };
    plan.plan_digest = prospective_event_maturity_plan_digest_v0(&plan);
    Ok(plan)
}

pub fn validate_prospective_event_maturity_plan_v0(
    plan: &ProspectiveEventMaturityPlanV0,
) -> Result<(), String> {
    if plan.plan_version != "prospective-event-maturity-plan-v0"
        || plan.event_id.is_empty()
        || plan.event_digest.is_empty()
        || plan.agent_id.is_empty()
        || plan.prediction_timestamp == 0
        || plan.maturity_timestamp <= plan.prediction_timestamp
        || plan.horizon_digest.is_empty()
        || plan.required_outcome_start_timestamp
            != plan
                .prediction_timestamp
                .saturating_add(PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0)
        || plan.required_finalized_row_count == 0
        || plan.required_outcome_end_timestamp != plan.maturity_timestamp
        || plan.required_outcome_end_timestamp
            != plan.required_outcome_start_timestamp.saturating_add(
                (plan.required_finalized_row_count.saturating_sub(1) as u64)
                    .saturating_mul(PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0),
            )
        || plan.label_policy_digest.is_empty()
        || plan.source_policy_digest.is_empty()
        || plan.plan_digest != prospective_event_maturity_plan_digest_v0(plan)
    {
        Err("prospective_maturity_plan_invalid".into())
    } else {
        Ok(())
    }
}

fn prospective_outcome_required_timestamps_v0(
    plans: &[ProspectiveEventMaturityPlanV0],
) -> Result<BTreeSet<u64>, String> {
    if plans.len() != 2 {
        return Err("prospective_maturity_plan_count_invalid".into());
    }
    let mut timestamps = BTreeSet::new();
    for plan in plans {
        validate_prospective_event_maturity_plan_v0(plan)?;
        for offset in 0..plan.required_finalized_row_count {
            timestamps.insert(
                plan.required_outcome_start_timestamp
                    .saturating_add(offset as u64 * PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0),
            );
        }
    }
    if timestamps.is_empty() || timestamps.len() > MAXIMUM_PROSPECTIVE_OUTCOME_RESPONSE_ROWS_V0 {
        return Err("prospective_outcome_request_range_invalid".into());
    }
    Ok(timestamps)
}

pub fn prospective_outcome_required_timestamp_set_v0(
    plans: &[ProspectiveEventMaturityPlanV0],
) -> Result<Vec<u64>, String> {
    Ok(prospective_outcome_required_timestamps_v0(plans)?
        .into_iter()
        .collect())
}

pub fn prospective_outcome_request_row_count_v0(
    plans: &[ProspectiveEventMaturityPlanV0],
) -> Result<usize, String> {
    Ok(prospective_outcome_required_timestamps_v0(plans)?.len())
}

pub fn prospective_outcome_request_readiness_v0(
    registration: &ProspectiveOneTimeOpeningRegistrationV0,
    plans: &[ProspectiveEventMaturityPlanV0],
    observed_timestamp: u64,
    event_integrity_valid: bool,
    prior_request_attempted: bool,
    outcome_evidence_present: bool,
) -> ProspectiveOutcomeRequestReadinessV0 {
    if validate_prospective_one_time_opening_registration_v0(registration, plans).is_err() {
        return ProspectiveOutcomeRequestReadinessV0::RegistrationInvalid;
    }
    if !event_integrity_valid {
        return ProspectiveOutcomeRequestReadinessV0::EventIntegrityInvalid;
    }
    if outcome_evidence_present {
        return ProspectiveOutcomeRequestReadinessV0::OutcomeEvidenceAlreadyPresent;
    }
    if prior_request_attempted {
        return ProspectiveOutcomeRequestReadinessV0::RequestAlreadyAttempted;
    }
    let mut momentum_boundary_reached = None;
    let mut risk_boundary_reached = None;
    for plan in plans {
        let finalization_boundary = match plan
            .required_outcome_end_timestamp
            .checked_add(PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0)
        {
            Some(value) => value,
            None => return ProspectiveOutcomeRequestReadinessV0::TechnicalFailure,
        };
        match plan.objective {
            LearnedAgentObjectiveV0::DirectionalMomentum => {
                if momentum_boundary_reached.is_some() {
                    return ProspectiveOutcomeRequestReadinessV0::TechnicalFailure;
                }
                momentum_boundary_reached = Some(observed_timestamp >= finalization_boundary);
            }
            LearnedAgentObjectiveV0::DownsideRisk => {
                if risk_boundary_reached.is_some() {
                    return ProspectiveOutcomeRequestReadinessV0::TechnicalFailure;
                }
                risk_boundary_reached = Some(observed_timestamp >= finalization_boundary);
            }
        }
    }
    match (momentum_boundary_reached, risk_boundary_reached) {
        (Some(true), Some(true)) => ProspectiveOutcomeRequestReadinessV0::ReadyForExplicitRequest,
        (Some(false), Some(true)) => {
            ProspectiveOutcomeRequestReadinessV0::AwaitingMomentumTimeMaturity
        }
        (Some(true), Some(false)) => ProspectiveOutcomeRequestReadinessV0::AwaitingRiskTimeMaturity,
        (Some(false), Some(false)) => {
            ProspectiveOutcomeRequestReadinessV0::AwaitingBothTimeMaturities
        }
        _ => ProspectiveOutcomeRequestReadinessV0::TechnicalFailure,
    }
}

pub fn pre_register_prospective_one_time_opening_v0(
    momentum_event: &LearnedProspectiveEventV0,
    risk_event: &LearnedProspectiveEventV0,
    momentum_horizon_rows: usize,
    risk_horizon_rows: usize,
    momentum_label_policy_digest: &str,
    risk_label_policy_digest: &str,
    outcome_source_policy_digest: &str,
    finalization_policy_digest: &str,
    metric_policy_digests: Vec<String>,
) -> Result<
    (
        ProspectiveOneTimeOpeningRegistrationV0,
        Vec<ProspectiveEventMaturityPlanV0>,
    ),
    String,
> {
    let momentum_plan = derive_prospective_event_maturity_plan_v0(
        momentum_event,
        momentum_horizon_rows,
        momentum_label_policy_digest,
        outcome_source_policy_digest,
    )?;
    let risk_plan = derive_prospective_event_maturity_plan_v0(
        risk_event,
        risk_horizon_rows,
        risk_label_policy_digest,
        outcome_source_policy_digest,
    )?;
    if momentum_plan.objective != LearnedAgentObjectiveV0::DirectionalMomentum
        || risk_plan.objective != LearnedAgentObjectiveV0::DownsideRisk
        || momentum_event.shared_raw_evidence_digest != risk_event.shared_raw_evidence_digest
        || finalization_policy_digest.is_empty()
        || metric_policy_digests.len() != 2
        || metric_policy_digests.iter().any(String::is_empty)
    {
        return Err("prospective_opening_registration_contract_invalid".into());
    }
    let plans = vec![momentum_plan, risk_plan];
    let maximum_response_rows = prospective_outcome_request_row_count_v0(&plans)?;
    let mut registration = ProspectiveOneTimeOpeningRegistrationV0 {
        registration_version: "prospective-one-time-opening-registration-v0".into(),
        momentum_event_digest: momentum_event.event_digest.clone(),
        risk_event_digest: risk_event.event_digest.clone(),
        maturity_plan_digests: plans.iter().map(|plan| plan.plan_digest.clone()).collect(),
        shared_raw_evidence_digest: momentum_event.shared_raw_evidence_digest.clone(),
        outcome_source_policy_digest: outcome_source_policy_digest.into(),
        finalization_policy_digest: finalization_policy_digest.into(),
        label_policy_digests: vec![
            momentum_label_policy_digest.into(),
            risk_label_policy_digest.into(),
        ],
        metric_policy_digests,
        maximum_future_requests: 1,
        maximum_concurrency: 1,
        maximum_retries: 0,
        maximum_response_rows,
        explicit_opening_authorization_required: true,
        one_time_opening_required: true,
        early_opening_forbidden: true,
        duplicate_opening_forbidden: true,
        interim_metrics_forbidden: true,
        network_execution_allowed_this_sprint: false,
        label_access_allowed_this_sprint: false,
        reward_application_allowed: false,
        registration_digest: String::new(),
    };
    registration.registration_digest =
        prospective_one_time_opening_registration_digest_v0(&registration);
    validate_prospective_one_time_opening_registration_v0(&registration, &plans)?;
    Ok((registration, plans))
}

pub fn validate_prospective_one_time_opening_registration_v0(
    registration: &ProspectiveOneTimeOpeningRegistrationV0,
    plans: &[ProspectiveEventMaturityPlanV0],
) -> Result<(), String> {
    let expected_rows = prospective_outcome_request_row_count_v0(plans)?;
    let plan_digests = plans
        .iter()
        .map(|plan| plan.plan_digest.clone())
        .collect::<Vec<_>>();
    let momentum_event_digest = plans
        .iter()
        .find(|plan| plan.objective == LearnedAgentObjectiveV0::DirectionalMomentum)
        .map(|plan| plan.event_digest.as_str());
    let risk_event_digest = plans
        .iter()
        .find(|plan| plan.objective == LearnedAgentObjectiveV0::DownsideRisk)
        .map(|plan| plan.event_digest.as_str());
    if registration.registration_version != "prospective-one-time-opening-registration-v0"
        || registration.momentum_event_digest.is_empty()
        || registration.risk_event_digest.is_empty()
        || registration.momentum_event_digest == registration.risk_event_digest
        || registration.maturity_plan_digests != plan_digests
        || momentum_event_digest != Some(registration.momentum_event_digest.as_str())
        || risk_event_digest != Some(registration.risk_event_digest.as_str())
        || registration.shared_raw_evidence_digest.is_empty()
        || registration.outcome_source_policy_digest.is_empty()
        || registration.finalization_policy_digest.is_empty()
        || registration.label_policy_digests.len() != 2
        || registration
            .label_policy_digests
            .iter()
            .any(String::is_empty)
        || registration.metric_policy_digests.len() != 2
        || registration
            .metric_policy_digests
            .iter()
            .any(String::is_empty)
        || registration.maximum_future_requests != 1
        || registration.maximum_concurrency != 1
        || registration.maximum_retries != 0
        || registration.maximum_response_rows != expected_rows
        || registration.maximum_response_rows == 0
        || registration.maximum_response_rows > MAXIMUM_PROSPECTIVE_OUTCOME_RESPONSE_ROWS_V0
        || !registration.explicit_opening_authorization_required
        || !registration.one_time_opening_required
        || !registration.early_opening_forbidden
        || !registration.duplicate_opening_forbidden
        || !registration.interim_metrics_forbidden
        || registration.network_execution_allowed_this_sprint
        || registration.label_access_allowed_this_sprint
        || registration.reward_application_allowed
        || registration.registration_digest
            != prospective_one_time_opening_registration_digest_v0(registration)
    {
        Err("prospective_opening_registration_invalid".into())
    } else {
        Ok(())
    }
}

pub fn write_prospective_one_time_opening_registration_v0(
    path: &Path,
    registration: &ProspectiveOneTimeOpeningRegistrationV0,
    plans: &[ProspectiveEventMaturityPlanV0],
) -> Result<(), String> {
    validate_prospective_one_time_opening_registration_v0(registration, plans)?;
    let parent = path
        .parent()
        .ok_or("prospective_opening_registration_storage_unavailable")?;
    fs::create_dir_all(parent)
        .map_err(|_| "prospective_opening_registration_storage_unavailable")?;
    let encoded = serde_json::to_vec(registration)
        .map_err(|_| "prospective_opening_registration_serialization_failed")?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, encoded)
        .map_err(|_| "prospective_opening_registration_storage_failed")?;
    fs::rename(temporary, path)
        .map_err(|_| "prospective_opening_registration_storage_failed".to_string())
}

pub fn read_prospective_one_time_opening_registration_v0(
    path: &Path,
) -> Result<ProspectiveOneTimeOpeningRegistrationV0, String> {
    serde_json::from_slice(
        &fs::read(path).map_err(|_| "prospective_opening_registration_unavailable")?,
    )
    .map_err(|_| "prospective_opening_registration_invalid".into())
}

pub fn assess_prospective_outcome_evidence_v0(
    plans: &[ProspectiveEventMaturityPlanV0],
    expected_series_id: &str,
    rows: &[ProspectiveOutcomeEvidenceRowV0],
) -> ProspectiveOutcomeEvidenceAssessmentV0 {
    let required_timestamps = match prospective_outcome_required_timestamps_v0(plans) {
        Ok(value) => value,
        Err(_) => {
            return ProspectiveOutcomeEvidenceAssessmentV0 {
                status: ProspectiveOutcomeEvidenceStatusV0::IntegrityInvalid,
                required_finalized_row_count: 0,
                observed_row_count: rows.len(),
                evidence_digest: None,
            };
        }
    };
    let required_finalized_row_count = required_timestamps.len();
    if rows.is_empty() {
        return ProspectiveOutcomeEvidenceAssessmentV0 {
            status: ProspectiveOutcomeEvidenceStatusV0::NoOutcomeRows,
            required_finalized_row_count,
            observed_row_count: 0,
            evidence_digest: None,
        };
    }
    if rows.iter().any(|row| row.series_id != expected_series_id) {
        return ProspectiveOutcomeEvidenceAssessmentV0 {
            status: ProspectiveOutcomeEvidenceStatusV0::WrongSeries,
            required_finalized_row_count,
            observed_row_count: rows.len(),
            evidence_digest: None,
        };
    }
    if rows.iter().any(|row| !row.finalized) {
        return ProspectiveOutcomeEvidenceAssessmentV0 {
            status: ProspectiveOutcomeEvidenceStatusV0::NonFinalizedRow,
            required_finalized_row_count,
            observed_row_count: rows.len(),
            evidence_digest: None,
        };
    }
    let mut observed = BTreeSet::new();
    let mut previous_timestamp = None;
    for row in rows {
        if previous_timestamp.is_some_and(|previous| row.timestamp < previous) {
            return ProspectiveOutcomeEvidenceAssessmentV0 {
                status: ProspectiveOutcomeEvidenceStatusV0::ChronologyInvalid,
                required_finalized_row_count,
                observed_row_count: rows.len(),
                evidence_digest: None,
            };
        }
        previous_timestamp = Some(row.timestamp);
        if !observed.insert(row.timestamp) {
            return ProspectiveOutcomeEvidenceAssessmentV0 {
                status: ProspectiveOutcomeEvidenceStatusV0::DuplicateRows,
                required_finalized_row_count,
                observed_row_count: rows.len(),
                evidence_digest: None,
            };
        }
        if !required_timestamps.contains(&row.timestamp) {
            return ProspectiveOutcomeEvidenceAssessmentV0 {
                status: ProspectiveOutcomeEvidenceStatusV0::ChronologyInvalid,
                required_finalized_row_count,
                observed_row_count: rows.len(),
                evidence_digest: None,
            };
        }
    }
    if observed != required_timestamps {
        let expected_prefix = required_timestamps
            .iter()
            .take(observed.len())
            .copied()
            .collect::<BTreeSet<_>>();
        return ProspectiveOutcomeEvidenceAssessmentV0 {
            status: if observed == expected_prefix {
                ProspectiveOutcomeEvidenceStatusV0::PartialOutcomeRows
            } else {
                ProspectiveOutcomeEvidenceStatusV0::MissingRequiredTimestamp
            },
            required_finalized_row_count,
            observed_row_count: rows.len(),
            evidence_digest: None,
        };
    }
    if rows.iter().any(|row| row.canonical_row_digest.is_empty()) {
        return ProspectiveOutcomeEvidenceAssessmentV0 {
            status: ProspectiveOutcomeEvidenceStatusV0::CompleteUnverified,
            required_finalized_row_count,
            observed_row_count: rows.len(),
            evidence_digest: None,
        };
    }
    ProspectiveOutcomeEvidenceAssessmentV0 {
        status: ProspectiveOutcomeEvidenceStatusV0::CompleteVerified,
        required_finalized_row_count,
        observed_row_count: rows.len(),
        evidence_digest: Some(stable_hash_string(&format!(
            "prospective-outcome-evidence-v0:{}:{:?}",
            expected_series_id,
            rows.iter()
                .map(|row| (&row.timestamp, &row.canonical_row_digest))
                .collect::<Vec<_>>(),
        ))),
    }
}

pub fn prospective_opening_readiness_v0(
    plan: &ProspectiveEventMaturityPlanV0,
    observed_timestamp: u64,
    evidence_status: ProspectiveOutcomeEvidenceStatusV0,
    event_integrity_valid: bool,
    challenge_valid: bool,
    label_open_count: usize,
) -> ProspectiveOpeningReadinessV0 {
    if validate_prospective_event_maturity_plan_v0(plan).is_err() || !event_integrity_valid {
        return ProspectiveOpeningReadinessV0::IntegrityInvalid;
    }
    if !challenge_valid {
        return ProspectiveOpeningReadinessV0::ChallengeInvalid;
    }
    if label_open_count > 0 {
        return ProspectiveOpeningReadinessV0::AlreadyOpened;
    }
    if observed_timestamp < plan.maturity_timestamp {
        return ProspectiveOpeningReadinessV0::AwaitingTimeMaturity;
    }
    match evidence_status {
        ProspectiveOutcomeEvidenceStatusV0::NoOutcomeRows
        | ProspectiveOutcomeEvidenceStatusV0::PartialOutcomeRows
        | ProspectiveOutcomeEvidenceStatusV0::MissingRequiredTimestamp => {
            ProspectiveOpeningReadinessV0::TimeMatureOutcomeRowsMissing
        }
        ProspectiveOutcomeEvidenceStatusV0::CompleteUnverified => {
            ProspectiveOpeningReadinessV0::OutcomeRowsPresentButUnverified
        }
        ProspectiveOutcomeEvidenceStatusV0::CompleteVerified => {
            ProspectiveOpeningReadinessV0::ReadyForExplicitOpening
        }
        ProspectiveOutcomeEvidenceStatusV0::DuplicateRows
        | ProspectiveOutcomeEvidenceStatusV0::NonFinalizedRow
        | ProspectiveOutcomeEvidenceStatusV0::WrongSeries
        | ProspectiveOutcomeEvidenceStatusV0::ChronologyInvalid
        | ProspectiveOutcomeEvidenceStatusV0::IntegrityInvalid => {
            ProspectiveOpeningReadinessV0::IntegrityInvalid
        }
    }
}

pub fn aggregate_prospective_opening_readiness_v0(
    readiness: &[ProspectiveOpeningReadinessV0],
) -> ProspectiveOpeningReadinessV0 {
    if readiness.len() != 2 {
        return ProspectiveOpeningReadinessV0::TechnicalFailure;
    }
    for status in readiness {
        if matches!(
            status,
            ProspectiveOpeningReadinessV0::IntegrityInvalid
                | ProspectiveOpeningReadinessV0::ChallengeInvalid
                | ProspectiveOpeningReadinessV0::AlreadyOpened
        ) {
            return *status;
        }
    }
    if readiness
        .iter()
        .any(|status| *status == ProspectiveOpeningReadinessV0::AwaitingTimeMaturity)
    {
        return ProspectiveOpeningReadinessV0::AwaitingTimeMaturity;
    }
    if readiness
        .iter()
        .any(|status| *status == ProspectiveOpeningReadinessV0::OutcomeRowsPresentButUnverified)
    {
        return ProspectiveOpeningReadinessV0::OutcomeRowsPresentButUnverified;
    }
    if readiness
        .iter()
        .any(|status| *status == ProspectiveOpeningReadinessV0::TimeMatureOutcomeRowsMissing)
    {
        return ProspectiveOpeningReadinessV0::TimeMatureOutcomeRowsMissing;
    }
    if readiness
        .iter()
        .all(|status| *status == ProspectiveOpeningReadinessV0::ReadyForExplicitOpening)
    {
        ProspectiveOpeningReadinessV0::ReadyForExplicitOpening
    } else {
        ProspectiveOpeningReadinessV0::TechnicalFailure
    }
}

pub fn validate_prospective_opening_authorization_v0(
    authorization: &ProspectiveOpeningAuthorizationV0,
    registration: &ProspectiveOneTimeOpeningRegistrationV0,
    readiness: ProspectiveOpeningReadinessV0,
    outcome_evidence_digest: &str,
    label_open_count: usize,
) -> Result<(), String> {
    let mut expected_events = vec![
        registration.momentum_event_digest.clone(),
        registration.risk_event_digest.clone(),
    ];
    expected_events.sort();
    let mut authorized_events = authorization.authorized_event_digests.clone();
    authorized_events.sort();
    if readiness != ProspectiveOpeningReadinessV0::ReadyForExplicitOpening
        || label_open_count != 0
        || authorization.authorization_version != "prospective-opening-authorization-v0"
        || authorization.opening_registration_digest != registration.registration_digest
        || authorized_events != expected_events
        || authorization.authorized_outcome_evidence_digest != outcome_evidence_digest
        || authorization.authorized_outcome_evidence_digest.is_empty()
        || !authorization.explicit_owner_authorization
        || !authorization.one_time_only
        || authorization.label_open_count_before != 0
        || authorization.authorization_digest
            != prospective_opening_authorization_digest_v0(authorization)
    {
        Err("prospective_opening_authorization_invalid".into())
    } else {
        Ok(())
    }
}

pub fn audit_sealed_prospective_events_v0(
    admission_registration: &ProspectiveExternalAdmissionRegistrationV0,
    external_capsule: &ProspectiveExternalRowCapsuleV0,
    momentum: &ProspectiveChallengeLocalStateV0,
    risk: &CycleRiskProspectiveLocalStateV0,
) -> Result<ProspectiveSealedEventAuditV0, String> {
    validate_prospective_challenge_local_state_v0(momentum)
        .map_err(|_| "prospective_maturity_momentum_journal_invalid")?;
    validate_cycle_risk_prospective_local_state_v0(risk)
        .map_err(|_| "prospective_maturity_risk_journal_invalid")?;
    validate_prospective_external_admission_registration_v0(
        admission_registration,
        momentum,
        &risk.capsule,
    )?;
    let shared = build_shared_prospective_raw_evidence_v0(
        admission_registration,
        external_capsule,
        ProspectiveRowAdmissionStatusV0::Admitted,
    )?;
    let momentum_validation = validate_momentum_shared_prospective_reference_v0(
        admission_registration,
        momentum,
        &shared,
    );
    let risk_validation =
        validate_risk_shared_prospective_reference_v0(admission_registration, risk, &shared);
    if !momentum_validation.independently_valid || !risk_validation.independently_valid {
        return Err("prospective_maturity_shared_reference_invalid".into());
    }
    let momentum_event = seal_external_prospective_event_v0(
        &momentum_validation,
        &shared,
        momentum,
        risk,
        ProspectiveOperationalOutcomeV0::ShadowAbstentionSupportUnavailable,
        Some("frozen_external_inference_support_unavailable".into()),
    )?;
    let risk_event = seal_external_prospective_event_v0(
        &risk_validation,
        &shared,
        momentum,
        risk,
        ProspectiveOperationalOutcomeV0::ShadowAbstentionSupportUnavailable,
        Some("frozen_external_inference_support_unavailable".into()),
    )?;
    let momentum_journal_event = momentum
        .journal
        .events
        .first()
        .ok_or("prospective_maturity_momentum_event_missing")?;
    if momentum.journal.events.len() != 1
        || momentum.vault.finalized_rows.len() != 1
        || momentum.vault.finalized_rows[0].timestamp_ms != shared.timestamp
        || momentum.vault.finalized_rows[0].canonical_row_digest != shared.canonical_row_digest
        || !momentum.vault.finalized_rows[0].finalized
        || momentum_journal_event.event_id != momentum_event.event_id
        || momentum_journal_event.prediction_timestamp_ms != momentum_event.prediction_timestamp
        || momentum_journal_event.required_label_maturity_timestamp_ms
            != momentum_event.maturity_timestamp
        || momentum_journal_event.input_evidence_digest != shared.reference_digest
        || momentum_journal_event.candidate_artifact_digest
            != momentum.capsule.candidate.artifact_digest
        || momentum_journal_event.comparator_artifact_digests
            != momentum
                .capsule
                .comparators
                .iter()
                .map(|value| value.artifact_digest.clone())
                .collect::<Vec<_>>()
        || momentum_journal_event.operational_outcome
            != ProspectiveShadowOutcomeV0::ShadowAbstainSupportUnavailable
        || momentum_journal_event.label_status != ProspectiveLabelStatusV0::AwaitingFutureRows
        || momentum.vault.labels_derived
        || momentum.vault.opened
    {
        return Err("prospective_maturity_momentum_event_invalid".into());
    }
    if risk.vault.row_count != 1
        || risk.journal.event_count != 1
        || risk.vault.admitted_row_timestamps != vec![shared.timestamp]
        || risk.vault.admitted_row_digests != vec![shared.canonical_row_digest.clone()]
        || risk.journal.sealed_event_timestamps != vec![risk_event.prediction_timestamp]
        || risk.journal.sealed_event_digests != vec![risk_event.event_digest.clone()]
        || risk.vault.labels_derived
        || risk.vault.opened
        || risk.journal.labels_accessed
        || risk.journal.evaluation_performed
    {
        return Err("prospective_maturity_risk_event_invalid".into());
    }
    Ok(ProspectiveSealedEventAuditV0 {
        shared_raw_evidence_digest: shared.reference_digest,
        momentum_event,
        risk_event,
    })
}

#[cfg(test)]
pub(crate) fn prospective_outcome_acquisition_test_registration_v0(
    outcome_source_policy_digest: &str,
) -> (
    ProspectiveOneTimeOpeningRegistrationV0,
    Vec<ProspectiveEventMaturityPlanV0>,
) {
    fn event(
        objective: LearnedAgentObjectiveV0,
        agent_id: &str,
        horizon_rows: usize,
    ) -> LearnedProspectiveEventV0 {
        let prediction_timestamp = 1_704_067_200_000;
        let mut event = LearnedProspectiveEventV0 {
            event_version: "learned-prospective-event-v0".into(),
            event_id: format!("outcome-acquisition-{agent_id}"),
            agent_id: agent_id.into(),
            objective,
            challenge_digest: format!("challenge-{agent_id}"),
            shared_raw_evidence_digest: "shared-evidence".into(),
            frozen_model_artifact_digests: vec![format!("artifact-{agent_id}")],
            input_digest: "shared-evidence".into(),
            support_status_digest: "support".into(),
            operational_outcome:
                ProspectiveOperationalOutcomeV0::ShadowAbstentionSupportUnavailable,
            abstention_reason: Some("frozen_support_unavailable".into()),
            prediction_timestamp,
            maturity_timestamp: prediction_timestamp
                + horizon_rows as u64 * PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0,
            horizon_digest: stable_hash_string(&format!(
                "external-prospective-horizon-v0:{objective:?}:{horizon_rows}"
            )),
            probability_bits_sealed: true,
            label_accessed: false,
            event_digest: String::new(),
        };
        event.event_digest = learned_prospective_event_digest_v0(&event);
        event
    }
    let momentum = event(
        LearnedAgentObjectiveV0::DirectionalMomentum,
        MOMENTUM_AGENT_ID_V0,
        1,
    );
    let risk = event(
        LearnedAgentObjectiveV0::DownsideRisk,
        CYCLE_RISK_SHADOW_AGENT_ID_V0,
        4,
    );
    pre_register_prospective_one_time_opening_v0(
        &momentum,
        &risk,
        1,
        4,
        "momentum-label",
        "risk-label",
        outcome_source_policy_digest,
        "finalized-contiguous-utc-daily",
        vec!["momentum-metric".into(), "risk-metric".into()],
    )
    .unwrap()
}

#[cfg(test)]
pub(crate) fn chair_shadow_test_observation_report_for_owner_review_v0()
-> ChairShadowObservationReportV0 {
    tests::chair_shadow_test_observation_report_for_owner_review_v0()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::ReasonCode,
        data::{
            AcquisitionMarketScope, DataLookback, DatasetKind, SnapshotProvenance,
            SnapshotQualitySummary, SnapshotSourceType,
        },
        league::{HistoricalOhlcvRow, HistoricalReplayDataset},
        model::historical_evidence::{
            BtcTemporalRegimeEvidenceResultV0, BtcTemporalRegimeRefV0, RegimeModelEvidenceOutcomeV0,
        },
        model::learning_campaign::{
            EarliestTemporalShiftStageV0, ProbabilityCollapseRootCauseV0,
            SupportGatedMomentumSeriesVerdictV0, WarmStartLockInStatusV0,
        },
    };

    fn v3_audit_fixture() -> (
        BtcTemporalRegimeEvidenceResultV0,
        BtcTemporalRegimeRefV0,
        crate::model::historical_evidence::BtcTemporalRegimeClosedResultV0,
    ) {
        let result = BtcTemporalRegimeEvidenceResultV0 {
            regime_id: "joint-scope-control".into(),
            row_count: 8,
            campaign_windows: 2,
            no_signal_windows: 1,
            selected_checkpoint_windows: 1,
            in_support_windows: 1,
            out_of_support_windows: 0,
            support_unavailable_windows: 0,
            validation_in_support_windows: 1,
            validation_out_of_support_windows: 0,
            validation_insufficient_windows: 0,
            validation_gate_unavailable_windows: 0,
            support_traces: vec![],
            earliest_shift_stage: EarliestTemporalShiftStageV0::InsufficientEvidence,
            temporal_root_cause: ProbabilityCollapseRootCauseV0::Unknown,
            frozen_representation_breach_count: 0,
            warm_start_status: WarmStartLockInStatusV0::WarmAndColdBothNoSignal,
            abstention_count: 1,
            accepted_predictive_versions: 0,
            final_verdict:
                SupportGatedMomentumSeriesVerdictV0::InSupportUsableSignalButLinearStrongerOnThisSeries,
            reason_codes: vec!["baseline_stronger".into()],
            campaign_config_digest: "campaign".into(),
            encoder_parameter_digest: "encoder".into(),
            report_digest: "open-report".into(),
        };
        let regime = BtcTemporalRegimeRefV0 {
            regime_id: result.regime_id.clone(),
            chronological_rank: 0,
            row_count: result.row_count,
            range_digest: "range".into(),
            pack_digest: "pack".into(),
        };
        let closed = close_btc_temporal_regime_result_v0(&result, regime.clone());
        (result, regime, closed)
    }

    fn v3_audit(
        result: &BtcTemporalRegimeEvidenceResultV0,
        regime: &BtcTemporalRegimeRefV0,
        closed: &crate::model::historical_evidence::BtcTemporalRegimeClosedResultV0,
    ) -> MomentumClosedResultContractAuditV3 {
        audit_momentum_closed_result_contract_v3(
            "joint-scope-control",
            result,
            regime,
            closed,
            "encoder".into(),
            "pack".into(),
            "derived".into(),
        )
    }

    fn scope(snapshot_id: &str, rows: &str, cutoff: u64) -> CanonicalRawObservationScopeV0 {
        CanonicalRawObservationScopeV0 {
            scope_version: "v0".into(),
            provider_id: "provider".into(),
            series_id: "series".into(),
            source_snapshot_id: snapshot_id.into(),
            source_snapshot_semantic_digest: "snapshot".into(),
            segmentation_policy_digest: "policy".into(),
            canonical_row_set_digest: rows.into(),
            canonical_row_order_digest: rows.into(),
            row_count: 2,
            first_timestamp: 1,
            last_timestamp: cutoff,
            information_cutoff_timestamp: cutoff,
            scope_kind: CanonicalObservationScopeKindV0::HistoricalRegimePack,
            scope_digest: rows.into(),
        }
    }

    fn joint_snapshot(rows: usize) -> DataSnapshot {
        let normalized_dataset = HistoricalReplayDataset {
            symbol: "BTC-KRW".into(),
            source: "approved-sanitized-history".into(),
            rows: (0..rows)
                .map(|index| {
                    let close = 100.0 + index as f64 * 0.12 + (index % 11) as f64 * 0.03;
                    HistoricalOhlcvRow {
                        symbol: "BTC-KRW".into(),
                        timestamp_ms: 1_700_000_000_000 + index as u64 * 60_000,
                        open: close - 0.05,
                        high: close + 0.15,
                        low: close - 0.2,
                        close,
                        volume: 1_000.0 + (index % 17) as f64,
                        trade_value: None,
                    }
                })
                .collect(),
            reason_codes: vec![],
        };
        let content_digest = historical_replay_dataset_digest_v0(&normalized_dataset);
        let first = normalized_dataset.rows.first().unwrap().timestamp_ms;
        let last = normalized_dataset.rows.last().unwrap().timestamp_ms;
        DataSnapshot {
            snapshot_id: snapshot_id_from_semantic_digest_v1(&content_digest),
            request_key: "btc-approved-history".into(),
            provider_id: "approved-provider".into(),
            dataset_kind: DatasetKind::DailyOhlcv,
            market_scope: AcquisitionMarketScope::BtcCrypto,
            symbols: vec!["BTC-KRW".into()],
            requested_lookback: DataLookback {
                bars: rows,
                start_timestamp_ms: Some(first),
                end_timestamp_ms: Some(last),
            },
            actual_start_timestamp_ms: Some(first),
            actual_end_timestamp_ms: Some(last),
            fetched_at_ms: last,
            normalized_at_ms: last,
            schema_version: 1,
            row_count: rows,
            quality_summary: SnapshotQualitySummary {
                accepted: true,
                row_count: rows,
                reason_codes: vec![],
            },
            content_digest,
            sanitized: true,
            read_only: true,
            normalized_dataset,
            provenance: SnapshotProvenance {
                provider_id: "approved-provider".into(),
                acquisition_request_id: "btc-approved-history".into(),
                fetch_receipt_id: "approved-receipt".into(),
                source_type: SnapshotSourceType::ApprovedReadOnlyProvider,
                sanitized: true,
                credential_free: true,
                reason_codes: vec![],
            },
            reason_codes: vec![ReasonCode::DataSnapshotImmutable],
        }
    }

    fn source_bound_test_opinion(
        agent_id: &str,
        objective: LearnedAgentObjectiveV0,
        result_digest: &str,
        rows: &[&str],
        anchors: &[&str],
        cutoff: u64,
    ) -> (LearnedAgentOpinionEnvelopeV1, LearnedAgentOpinionSealV1) {
        let registration = SourceBoundOpinionProtocolRegistrationV1::pre_registered();
        let mut source = LearnedAgentSourceResultReferenceV1 {
            agent_id: agent_id.into(),
            objective,
            source_snapshot_id: "snapshot".into(),
            source_snapshot_digest: "snapshot-digest".into(),
            source_result_kind: match objective {
                LearnedAgentObjectiveV0::DirectionalMomentum => {
                    SourceResultKindV1::MomentumHistoricalRegimeResult
                }
                LearnedAgentObjectiveV0::DownsideRisk => {
                    SourceResultKindV1::CycleRiskHistoricalRegimeResult
                }
            },
            source_result_digest_v1: result_digest.into(),
            source_checkpoint_digest_v1: format!("checkpoint-{result_digest}"),
            source_frozen_pack_digest: format!("pack-{result_digest}"),
            source_model_version_id: None,
            source_model_artifact_digest: format!("model-{result_digest}"),
            canonical_raw_scope_digest_v1: format!("scope-{result_digest}"),
            canonical_raw_row_identity_digests_v1: rows
                .iter()
                .map(|value| (*value).into())
                .collect(),
            information_cutoff_timestamp: cutoff,
            effective_anchor_scope_digest_v1: format!("anchors-{result_digest}"),
            effective_anchor_digests_v1: anchors.iter().map(|value| (*value).into()).collect(),
            forecast_scope_digest_v1: "forecast".into(),
            reference_digest_v1: String::new(),
        };
        source.reference_digest_v1 = source_reference_digest_v1(&source);
        let membership = SourceResultMembershipProofV1 {
            result_digest_v1: source.source_result_digest_v1.clone(),
            parent_report_digest: "report".into(),
            immutable_member: true,
            snapshot_matches: true,
            pack_matches: true,
            scope_matches: true,
            anchors_match: true,
            objective_matches: true,
            agent_matches: true,
            all_invariants_pass: true,
            proof_digest_v1: "proof".into(),
        };
        let mut opinion = create_source_bound_opinion_v1(
            source,
            &membership,
            cutoff,
            "test-doctrine",
            &registration,
        )
        .unwrap();
        let seal = source_bound_seal_v1(
            &opinion.opinion_id,
            &opinion.opinion_digest_v1,
            &opinion.source_result,
            &registration,
            &opinion.authority,
        )
        .unwrap();
        opinion.sealed = true;
        (opinion, seal)
    }

    fn reward_contract(
        objective: LearnedAgentObjectiveV0,
        challenge: &str,
    ) -> LearnedProspectiveContractV0 {
        let mut contract = LearnedProspectiveContractV0 {
            objective,
            agent_id: match objective {
                LearnedAgentObjectiveV0::DirectionalMomentum => MOMENTUM_AGENT_ID_V0.into(),
                LearnedAgentObjectiveV0::DownsideRisk => CYCLE_RISK_SHADOW_AGENT_ID_V0.into(),
            },
            challenge_digest: challenge.into(),
            model_artifact_digest: format!("model-{challenge}"),
            prediction_horizon_digest: format!("horizon-{challenge}"),
            cutoff_exclusive_timestamp: 100,
            sealed_shadow_only: true,
            contract_digest: String::new(),
        };
        contract.contract_digest = learned_contract_digest_v0(&contract);
        contract
    }

    fn reward_gate() -> LearnedRewardSampleGateV0 {
        let mut gate = LearnedRewardSampleGateV0 {
            minimum_mature_events: 1,
            minimum_support_qualified_events: 1,
            minimum_regime_coverage: 1,
            maximum_integrity_failures: 0,
            gate_digest: String::new(),
        };
        gate.gate_digest = learned_reward_gate_digest_v0(&gate);
        gate
    }

    fn reward_registration() -> LearnedRewardEligibilityRegistrationV0 {
        pre_register_learned_reward_eligibility_v0(&LearnedRewardEligibilityRegistrationInputV0 {
            momentum: reward_contract(LearnedAgentObjectiveV0::DirectionalMomentum, "momentum"),
            cycle_risk: reward_contract(LearnedAgentObjectiveV0::DownsideRisk, "risk"),
            attribution_policy_digest: "attribution".into(),
            maturity_policy_digest: "maturity".into(),
            sample_gate_policy_digest: reward_gate().gate_digest,
            objective_mapping_policy_digest: "mapping".into(),
            integrity_policy_digest: "integrity".into(),
        })
        .unwrap()
    }

    fn reward_event(
        contract: &LearnedProspectiveContractV0,
    ) -> LearnedProspectiveEventAttributionV0 {
        let mut event = LearnedProspectiveEventAttributionV0 {
            attribution_version: "learned-prospective-event-attribution-v0".into(),
            event_id: "prospective-event-1".into(),
            event_digest: "event".into(),
            agent_id: contract.agent_id.clone(),
            objective: contract.objective,
            opinion_id: "sealed-opinion".into(),
            opinion_digest: "opinion".into(),
            opinion_seal_digest: "seal".into(),
            challenge_digest: contract.challenge_digest.clone(),
            model_artifact_digest: contract.model_artifact_digest.clone(),
            raw_evidence_digest: "raw-evidence".into(),
            event_timestamp: 101,
            maturity_timestamp: 102,
            prediction_horizon_digest: contract.prediction_horizon_digest.clone(),
            support_status_digest: "support".into(),
            attribution_digest: String::new(),
        };
        event.attribution_digest = learned_attribution_digest_v0(&event);
        event
    }

    #[test]
    fn learned_reward_registration_is_immutable_and_mutation_forbidden() {
        let registration = reward_registration();
        assert!(validate_learned_reward_eligibility_registration_v0(&registration).is_ok());
        assert!(registration.retrospective_evidence_forbidden);
        assert!(registration.owner_input_forbidden);
        assert!(registration.reward_application_forbidden);
        assert!(registration.voice_mutation_forbidden);
        assert!(registration.cooldown_mutation_forbidden);
        assert!(registration.promotion_mutation_forbidden);
    }

    #[test]
    fn learned_reward_registration_rejects_changed_boolean() {
        let mut registration = reward_registration();
        registration.owner_input_forbidden = false;
        assert!(validate_learned_reward_eligibility_registration_v0(&registration).is_err());
    }

    #[test]
    fn learned_contract_requires_exact_agent_and_sealed_status() {
        let mut contract =
            reward_contract(LearnedAgentObjectiveV0::DirectionalMomentum, "momentum");
        contract.agent_id = CYCLE_RISK_SHADOW_AGENT_ID_V0.into();
        assert!(validate_learned_prospective_contract_v0(&contract).is_err());
        let mut contract =
            reward_contract(LearnedAgentObjectiveV0::DirectionalMomentum, "momentum");
        contract.sealed_shadow_only = false;
        assert!(validate_learned_prospective_contract_v0(&contract).is_err());
    }

    #[test]
    fn learned_reward_sample_gate_is_derived_and_nonzero() {
        assert!(validate_learned_reward_sample_gate_v0(&reward_gate()).is_ok());
        let mut gate = reward_gate();
        gate.minimum_mature_events = 0;
        assert!(validate_learned_reward_sample_gate_v0(&gate).is_err());
    }

    #[test]
    fn prospective_attribution_requires_sealed_opinion_and_exact_contract() {
        let contract = reward_contract(LearnedAgentObjectiveV0::DirectionalMomentum, "momentum");
        let event = reward_event(&contract);
        assert!(
            validate_learned_prospective_event_attribution_v0(&event, &contract, &BTreeSet::new())
                .is_ok()
        );
        let mut unsealed = event.clone();
        unsealed.opinion_seal_digest.clear();
        unsealed.attribution_digest = learned_attribution_digest_v0(&unsealed);
        assert!(
            validate_learned_prospective_event_attribution_v0(
                &unsealed,
                &contract,
                &BTreeSet::new()
            )
            .is_err()
        );
        let risk = reward_contract(LearnedAgentObjectiveV0::DownsideRisk, "risk");
        assert!(
            validate_learned_prospective_event_attribution_v0(&event, &risk, &BTreeSet::new())
                .is_err()
        );
    }

    #[test]
    fn prospective_attribution_rejects_duplicate_and_retroactive_event() {
        let contract = reward_contract(LearnedAgentObjectiveV0::DirectionalMomentum, "momentum");
        let event = reward_event(&contract);
        let ids = BTreeSet::from([event.event_id.clone()]);
        assert!(
            validate_learned_prospective_event_attribution_v0(&event, &contract, &ids).is_err()
        );
        let mut retroactive = event.clone();
        retroactive.event_timestamp = 100;
        retroactive.attribution_digest = learned_attribution_digest_v0(&retroactive);
        assert!(
            validate_learned_prospective_event_attribution_v0(
                &retroactive,
                &contract,
                &BTreeSet::new()
            )
            .is_err()
        );
    }

    #[test]
    fn outcome_opening_requires_maturity_identity_and_authorization() {
        let base = LearnedOutcomeOpeningRequestV0 {
            event_timestamp: 101,
            maturity_timestamp: 102,
            observed_timestamp: 103,
            required_finalized_rows_present: true,
            event_identity_matches: true,
            challenge_valid: true,
            explicit_authorization: true,
            already_opened: false,
        };
        assert_eq!(
            learned_outcome_maturity_status_v0(&base),
            LearnedOutcomeMaturityStatusV0::MatureOpenedOnce
        );
        let mut early = base.clone();
        early.observed_timestamp = 101;
        assert_eq!(
            learned_outcome_maturity_status_v0(&early),
            LearnedOutcomeMaturityStatusV0::OpenedEarlyInvalid
        );
        let mut duplicate = base.clone();
        duplicate.already_opened = true;
        assert_eq!(
            learned_outcome_maturity_status_v0(&duplicate),
            LearnedOutcomeMaturityStatusV0::DuplicateOpeningInvalid
        );
        let mut no_auth = base;
        no_auth.explicit_authorization = false;
        assert_eq!(
            learned_outcome_maturity_status_v0(&no_auth),
            LearnedOutcomeMaturityStatusV0::IntegrityInvalid
        );
    }

    fn momentum_outcome_record(
        event: &LearnedProspectiveEventAttributionV0,
    ) -> LearnedProspectiveOutcomeRecordV0 {
        LearnedProspectiveOutcomeRecordV0 {
            record_version: String::new(),
            event_id: event.event_id.clone(),
            attribution_digest: event.attribution_digest.clone(),
            agent_id: event.agent_id.clone(),
            objective: event.objective,
            payload: LearnedProspectiveOutcomePayloadV0::Momentum(MomentumProspectiveOutcomeV0 {
                directional_label_correct: true,
                support_qualified_brier_improved: true,
                calibration_improved: true,
                high_confidence_error: false,
                baseline_beaten: true,
                abstention: LearnedAbstentionAttributionV0::JustifiedCapitalProtection,
                probability_collapse: false,
                support_qualified: true,
                regime_digest: "regime-a".into(),
            }),
            maturity_status: LearnedOutcomeMaturityStatusV0::MatureOpenedOnce,
            challenge_valid: true,
            integrity_valid: true,
            retrospective_evidence: false,
            owner_influence: false,
            outcome_digest: String::new(),
        }
    }

    fn opened_momentum_ledger() -> (
        LearnedRewardEligibilityRegistrationV0,
        LearnedRewardSampleGateV0,
        LearnedProspectiveOutcomeLedgerV0,
    ) {
        let registration = reward_registration();
        let gate = reward_gate();
        let contract = reward_contract(LearnedAgentObjectiveV0::DirectionalMomentum, "momentum");
        let event = reward_event(&contract);
        let mut ledger = new_learned_prospective_outcome_ledger_v0(&registration).unwrap();
        append_learned_prospective_event_attribution_v0(
            &mut ledger,
            &registration,
            event.clone(),
            &contract,
        )
        .unwrap();
        append_learned_matured_outcome_v0(
            &mut ledger,
            &registration,
            momentum_outcome_record(&event),
        )
        .unwrap();
        (registration, gate, ledger)
    }

    fn replace_outcome_for_status(
        ledger: &mut LearnedProspectiveOutcomeLedgerV0,
        change: impl FnOnce(&mut LearnedProspectiveOutcomeRecordV0),
    ) {
        change(ledger.matured_outcomes.first_mut().unwrap());
        let outcome = ledger.matured_outcomes.first_mut().unwrap();
        outcome.outcome_digest = learned_outcome_digest_v0(outcome);
        ledger.ledger_digest = learned_outcome_ledger_digest_v0(ledger);
    }

    #[test]
    fn current_empty_ledger_derives_no_prospective_outcomes() {
        let registration = reward_registration();
        let eligibility = derive_learned_reward_eligibility_v0(
            &registration,
            &reward_gate(),
            &new_learned_prospective_outcome_ledger_v0(&registration).unwrap(),
            LearnedAgentObjectiveV0::DirectionalMomentum,
        )
        .unwrap();
        assert_eq!(
            eligibility.eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleNoProspectiveOutcomes
        );
    }

    #[test]
    fn prospective_event_without_opened_label_awaits_maturity() {
        let registration = reward_registration();
        let contract = reward_contract(LearnedAgentObjectiveV0::DirectionalMomentum, "momentum");
        let mut ledger = new_learned_prospective_outcome_ledger_v0(&registration).unwrap();
        append_learned_prospective_event_attribution_v0(
            &mut ledger,
            &registration,
            reward_event(&contract),
            &contract,
        )
        .unwrap();
        let eligibility = derive_learned_reward_eligibility_v0(
            &registration,
            &reward_gate(),
            &ledger,
            LearnedAgentObjectiveV0::DirectionalMomentum,
        )
        .unwrap();
        assert_eq!(
            eligibility.eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleAwaitingMaturity
        );
    }

    #[test]
    fn mature_eligible_fixture_produces_compute_only_candidate() {
        let (registration, gate, ledger) = opened_momentum_ledger();
        let eligibility = derive_learned_reward_eligibility_v0(
            &registration,
            &gate,
            &ledger,
            LearnedAgentObjectiveV0::DirectionalMomentum,
        )
        .unwrap();
        assert_eq!(
            eligibility.eligibility_status,
            LearnedRewardEligibilityStatusV0::EligibleForCandidateComputation
        );
        let candidate = learned_reward_input_candidate_v0(&eligibility, &ledger).unwrap();
        assert!(candidate.eligible_for_existing_reward_compute);
        assert!(!candidate.eligible_for_application);
        assert_eq!(ledger.reward_candidate_count, 0);
        assert_eq!(ledger.reward_apply_count, 0);
    }

    #[test]
    fn objective_payloads_cannot_swap() {
        let (registration, _, mut ledger) = opened_momentum_ledger();
        replace_outcome_for_status(&mut ledger, |outcome| {
            outcome.payload =
                LearnedProspectiveOutcomePayloadV0::CycleRisk(CycleRiskProspectiveOutcomeV0 {
                    downside_label_correct: true,
                    support_qualified_brier_improved: true,
                    calibration_improved: true,
                    high_confidence_false_negative: false,
                    correct_elevated_risk_warning: true,
                    false_permanent_alarm: false,
                    abstention: LearnedAbstentionAttributionV0::CorrectUncertainty,
                    probability_collapse: false,
                    support_qualified: true,
                    regime_digest: "regime-a".into(),
                });
        });
        let eligibility = derive_learned_reward_eligibility_v0(
            &registration,
            &reward_gate(),
            &ledger,
            LearnedAgentObjectiveV0::DirectionalMomentum,
        )
        .unwrap();
        assert_eq!(
            eligibility.eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleUnsupportedObjective
        );
    }

    #[test]
    fn duplicate_opening_is_rejected_without_mutating_ledger() {
        let (registration, _, mut ledger) = opened_momentum_ledger();
        let before = ledger.clone();
        let event = ledger.event_attributions.first().unwrap().clone();
        assert!(
            append_learned_matured_outcome_v0(
                &mut ledger,
                &registration,
                momentum_outcome_record(&event),
            )
            .is_err()
        );
        assert_eq!(ledger, before);
    }

    #[test]
    fn ledger_reopens_deterministically_after_append() {
        let (registration, _, ledger) = opened_momentum_ledger();
        assert!(validate_learned_prospective_outcome_ledger_v0(&ledger, &registration).is_ok());
        let reopened = ledger.clone();
        assert_eq!(reopened.ledger_digest, ledger.ledger_digest);
    }

    #[test]
    fn retrospective_owner_and_integrity_evidence_are_ineligible() {
        let (registration, gate, mut ledger) = opened_momentum_ledger();
        replace_outcome_for_status(&mut ledger, |outcome| outcome.retrospective_evidence = true);
        assert_eq!(
            derive_learned_reward_eligibility_v0(
                &registration,
                &gate,
                &ledger,
                LearnedAgentObjectiveV0::DirectionalMomentum,
            )
            .unwrap()
            .eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleRetrospectiveEvidence
        );
        replace_outcome_for_status(&mut ledger, |outcome| {
            outcome.retrospective_evidence = false;
            outcome.owner_influence = true;
        });
        assert_eq!(
            derive_learned_reward_eligibility_v0(
                &registration,
                &gate,
                &ledger,
                LearnedAgentObjectiveV0::DirectionalMomentum,
            )
            .unwrap()
            .eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleOwnerInfluence
        );
        replace_outcome_for_status(&mut ledger, |outcome| {
            outcome.owner_influence = false;
            outcome.integrity_valid = false;
        });
        assert_eq!(
            derive_learned_reward_eligibility_v0(
                &registration,
                &gate,
                &ledger,
                LearnedAgentObjectiveV0::DirectionalMomentum,
            )
            .unwrap()
            .eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleIntegrityFailure
        );
    }

    #[test]
    fn early_and_duplicate_statuses_are_never_eligible() {
        let (registration, gate, mut ledger) = opened_momentum_ledger();
        replace_outcome_for_status(&mut ledger, |outcome| {
            outcome.maturity_status = LearnedOutcomeMaturityStatusV0::OpenedEarlyInvalid;
        });
        assert_eq!(
            derive_learned_reward_eligibility_v0(
                &registration,
                &gate,
                &ledger,
                LearnedAgentObjectiveV0::DirectionalMomentum,
            )
            .unwrap()
            .eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleEarlyLabelAccess
        );
        replace_outcome_for_status(&mut ledger, |outcome| {
            outcome.maturity_status = LearnedOutcomeMaturityStatusV0::DuplicateOpeningInvalid;
        });
        assert_eq!(
            derive_learned_reward_eligibility_v0(
                &registration,
                &gate,
                &ledger,
                LearnedAgentObjectiveV0::DirectionalMomentum,
            )
            .unwrap()
            .eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleDuplicateOpening
        );
    }

    #[test]
    fn sample_and_regime_gates_are_derived_not_hard_coded() {
        let (registration, _, ledger) = opened_momentum_ledger();
        let mut samples = reward_gate();
        samples.minimum_mature_events = 2;
        samples.gate_digest = learned_reward_gate_digest_v0(&samples);
        assert_eq!(
            derive_learned_reward_eligibility_v0(
                &registration,
                &samples,
                &ledger,
                LearnedAgentObjectiveV0::DirectionalMomentum,
            )
            .unwrap()
            .eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleMinimumSamples
        );
        let mut regimes = reward_gate();
        regimes.minimum_regime_coverage = 2;
        regimes.gate_digest = learned_reward_gate_digest_v0(&regimes);
        assert_eq!(
            derive_learned_reward_eligibility_v0(
                &registration,
                &regimes,
                &ledger,
                LearnedAgentObjectiveV0::DirectionalMomentum,
            )
            .unwrap()
            .eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleInsufficientRegimeCoverage
        );
    }

    #[test]
    fn candidate_carries_typed_reward_signals_without_application() {
        let (registration, gate, ledger) = opened_momentum_ledger();
        let eligibility = derive_learned_reward_eligibility_v0(
            &registration,
            &gate,
            &ledger,
            LearnedAgentObjectiveV0::DirectionalMomentum,
        )
        .unwrap();
        let candidate = learned_reward_input_candidate_v0(&eligibility, &ledger).unwrap();
        assert!(
            candidate
                .typed_reward_signals
                .contains(&LearnedRewardSignalV0::CalibratedProspectiveAccuracy)
        );
        assert!(
            candidate
                .typed_reward_signals
                .contains(&LearnedRewardSignalV0::JustifiedCapitalProtection)
        );
        assert!(!candidate.eligible_for_application);
    }

    #[test]
    fn direct_matured_outcome_append_rejects_owner_or_retroactive_flags() {
        let registration = reward_registration();
        let contract = reward_contract(LearnedAgentObjectiveV0::DirectionalMomentum, "momentum");
        let event = reward_event(&contract);
        let mut ledger = new_learned_prospective_outcome_ledger_v0(&registration).unwrap();
        append_learned_prospective_event_attribution_v0(
            &mut ledger,
            &registration,
            event.clone(),
            &contract,
        )
        .unwrap();
        let mut record = momentum_outcome_record(&event);
        record.owner_influence = true;
        assert!(append_learned_matured_outcome_v0(&mut ledger, &registration, record).is_err());
        assert_eq!(ledger.label_open_count, 0);
        assert_eq!(ledger.reward_apply_count, 0);
    }

    #[test]
    fn registration_digest_is_deterministic_for_same_contracts() {
        assert_eq!(
            reward_registration().registration_digest,
            reward_registration().registration_digest
        );
    }

    #[test]
    fn attribution_digest_detects_event_tampering() {
        let contract = reward_contract(LearnedAgentObjectiveV0::DirectionalMomentum, "momentum");
        let mut event = reward_event(&contract);
        event.raw_evidence_digest = "changed".into();
        assert!(
            validate_learned_prospective_event_attribution_v0(&event, &contract, &BTreeSet::new(),)
                .is_err()
        );
    }

    #[test]
    fn challenge_invalidation_is_derived_as_ineligible() {
        let (registration, gate, mut ledger) = opened_momentum_ledger();
        replace_outcome_for_status(&mut ledger, |outcome| outcome.challenge_valid = false);
        assert_eq!(
            derive_learned_reward_eligibility_v0(
                &registration,
                &gate,
                &ledger,
                LearnedAgentObjectiveV0::DirectionalMomentum,
            )
            .unwrap()
            .eligibility_status,
            LearnedRewardEligibilityStatusV0::IneligibleChallengeInvalidated
        );
    }

    #[test]
    fn momentum_high_confidence_loss_becomes_typed_penalty() {
        let (registration, gate, mut ledger) = opened_momentum_ledger();
        replace_outcome_for_status(&mut ledger, |outcome| {
            let LearnedProspectiveOutcomePayloadV0::Momentum(value) = &mut outcome.payload else {
                panic!("fixture must remain momentum");
            };
            value.high_confidence_error = true;
        });
        let eligibility = derive_learned_reward_eligibility_v0(
            &registration,
            &gate,
            &ledger,
            LearnedAgentObjectiveV0::DirectionalMomentum,
        )
        .unwrap();
        let candidate = learned_reward_input_candidate_v0(&eligibility, &ledger).unwrap();
        assert!(
            candidate
                .typed_penalty_signals
                .contains(&LearnedPenaltySignalV0::HighConfidenceProspectiveError)
        );
    }

    #[test]
    fn momentum_probability_collapse_becomes_typed_penalty() {
        let (registration, gate, mut ledger) = opened_momentum_ledger();
        replace_outcome_for_status(&mut ledger, |outcome| {
            let LearnedProspectiveOutcomePayloadV0::Momentum(value) = &mut outcome.payload else {
                panic!("fixture must remain momentum");
            };
            value.probability_collapse = true;
        });
        let eligibility = derive_learned_reward_eligibility_v0(
            &registration,
            &gate,
            &ledger,
            LearnedAgentObjectiveV0::DirectionalMomentum,
        )
        .unwrap();
        let candidate = learned_reward_input_candidate_v0(&eligibility, &ledger).unwrap();
        assert!(
            candidate
                .typed_penalty_signals
                .contains(&LearnedPenaltySignalV0::ProbabilityCollapse)
        );
    }

    #[test]
    fn candidate_cannot_be_created_from_insufficient_samples() {
        let registration = reward_registration();
        let gate = reward_gate();
        let ledger = new_learned_prospective_outcome_ledger_v0(&registration).unwrap();
        let eligibility = derive_learned_reward_eligibility_v0(
            &registration,
            &gate,
            &ledger,
            LearnedAgentObjectiveV0::DirectionalMomentum,
        )
        .unwrap();
        assert!(learned_reward_input_candidate_v0(&eligibility, &ledger).is_err());
    }

    #[test]
    fn all_abstention_classifications_remain_objective_local() {
        let values = [
            LearnedAbstentionAttributionV0::JustifiedCapitalProtection,
            LearnedAbstentionAttributionV0::CorrectUncertainty,
            LearnedAbstentionAttributionV0::MissedMaterialOpportunity,
            LearnedAbstentionAttributionV0::FailedToWarnMaterialRisk,
            LearnedAbstentionAttributionV0::NeutralUninformative,
            LearnedAbstentionAttributionV0::NotYetEvaluable,
        ];
        assert_eq!(values.len(), 6);
        assert_ne!(values[0], values[5]);
    }

    #[test]
    fn cycle_risk_false_negative_is_a_distinct_penalty_signal() {
        let registration = reward_registration();
        let contract = reward_contract(LearnedAgentObjectiveV0::DownsideRisk, "risk");
        let mut event = reward_event(&contract);
        event.event_id = "risk-event".into();
        event.event_digest = "risk-event-digest".into();
        event.attribution_digest = learned_attribution_digest_v0(&event);
        let mut ledger = new_learned_prospective_outcome_ledger_v0(&registration).unwrap();
        append_learned_prospective_event_attribution_v0(
            &mut ledger,
            &registration,
            event.clone(),
            &contract,
        )
        .unwrap();
        append_learned_matured_outcome_v0(
            &mut ledger,
            &registration,
            LearnedProspectiveOutcomeRecordV0 {
                record_version: String::new(),
                event_id: event.event_id.clone(),
                attribution_digest: event.attribution_digest.clone(),
                agent_id: event.agent_id.clone(),
                objective: LearnedAgentObjectiveV0::DownsideRisk,
                payload: LearnedProspectiveOutcomePayloadV0::CycleRisk(
                    CycleRiskProspectiveOutcomeV0 {
                        downside_label_correct: false,
                        support_qualified_brier_improved: false,
                        calibration_improved: false,
                        high_confidence_false_negative: true,
                        correct_elevated_risk_warning: false,
                        false_permanent_alarm: false,
                        abstention: LearnedAbstentionAttributionV0::FailedToWarnMaterialRisk,
                        probability_collapse: false,
                        support_qualified: true,
                        regime_digest: "risk-regime".into(),
                    },
                ),
                maturity_status: LearnedOutcomeMaturityStatusV0::MatureOpenedOnce,
                challenge_valid: true,
                integrity_valid: true,
                retrospective_evidence: false,
                owner_influence: false,
                outcome_digest: String::new(),
            },
        )
        .unwrap();
        let eligibility = derive_learned_reward_eligibility_v0(
            &registration,
            &reward_gate(),
            &ledger,
            LearnedAgentObjectiveV0::DownsideRisk,
        )
        .unwrap();
        let candidate = learned_reward_input_candidate_v0(&eligibility, &ledger).unwrap();
        assert!(
            candidate
                .typed_penalty_signals
                .contains(&LearnedPenaltySignalV0::HighConfidenceRiskFalseNegative)
        );
    }

    #[test]
    fn ledger_never_records_reward_application_in_this_bridge() {
        let (registration, _, ledger) = opened_momentum_ledger();
        assert!(validate_learned_prospective_outcome_ledger_v0(&ledger, &registration).is_ok());
        assert_eq!(ledger.reward_apply_count, 0);
        assert_eq!(ledger.reward_candidate_count, 0);
    }

    #[test]
    fn mature_unopened_status_is_not_an_outcome_record() {
        let request = LearnedOutcomeOpeningRequestV0 {
            event_timestamp: 101,
            maturity_timestamp: 102,
            observed_timestamp: 102,
            required_finalized_rows_present: true,
            event_identity_matches: true,
            challenge_valid: true,
            explicit_authorization: true,
            already_opened: false,
        };
        assert_eq!(
            learned_outcome_maturity_status_v0(&request),
            LearnedOutcomeMaturityStatusV0::MatureUnopened
        );
    }
    #[test]
    fn scope_identity_changes_when_row_content_or_order_changes() {
        let ordered = vec!["a".to_string(), "b".to_string()];
        let reordered = vec!["b".to_string(), "a".to_string()];
        let changed = vec!["a".to_string(), "c".to_string()];
        assert_ne!(digest(&ordered), digest(&reordered));
        assert_ne!(digest(&ordered), digest(&changed));
    }

    #[test]
    fn lineage_requires_rows_and_cutoff_not_scope_names() {
        let same_rows_other_container =
            canonical_scope_lineage_proof_v0(&scope("left", "rows", 2), &scope("right", "rows", 2));
        assert_eq!(
            same_rows_other_container.lineage_status,
            CanonicalScopeLineageStatusV0::CertifiedEquivalentRows
        );
        let changed_cutoff =
            canonical_scope_lineage_proof_v0(&scope("left", "rows", 2), &scope("right", "rows", 3));
        assert_eq!(
            changed_cutoff.lineage_status,
            CanonicalScopeLineageStatusV0::DifferentInformationCutoff
        );
        let changed_rows = canonical_scope_lineage_proof_v0(
            &scope("left", "rows-a", 2),
            &scope("right", "rows-b", 2),
        );
        assert_eq!(
            changed_rows.lineage_status,
            CanonicalScopeLineageStatusV0::DifferentRows
        );
    }

    #[test]
    fn v1_canonical_row_identity_uses_exact_bits_and_optional_tag() {
        let base = CanonicalHistoricalRowIdentityV1 {
            provider_id: "provider".into(),
            series_id: "series".into(),
            timestamp_ms: 7,
            open_bits: 1.0f64.to_bits(),
            high_bits: 2.0f64.to_bits(),
            low_bits: 0.5f64.to_bits(),
            close_bits: 1.5f64.to_bits(),
            volume_bits: 9.0f64.to_bits(),
            trade_value_bits: None,
            row_digest_v1: String::new(),
        };
        let mut changed = base.clone();
        changed.trade_value_bits = Some(0);
        assert_eq!(
            canonical_semantic_digest_v1(&base),
            canonical_semantic_digest_v1(&base)
        );
        assert_ne!(
            canonical_semantic_digest_v1(&base),
            canonical_semantic_digest_v1(&changed)
        );
    }

    #[test]
    fn v1_encoder_preserves_vector_order_and_enum_tags() {
        let ordered = strings_digest_v1("test", &["a".into(), "b".into()]);
        let reordered = strings_digest_v1("test", &["b".into(), "a".into()]);
        assert_ne!(ordered, reordered);
        assert_ne!(
            verdict_tag_v1(crate::model::CycleRiskShadowVerdictV0::PositiveEvidence),
            verdict_tag_v1(crate::model::CycleRiskShadowVerdictV0::ProbabilityCollapse)
        );
    }

    #[test]
    fn source_bound_mapping_records_partial_rows_without_making_them_comparable() {
        let momentum = vec![source_bound_test_opinion(
            "momentum",
            LearnedAgentObjectiveV0::DirectionalMomentum,
            "momentum-result",
            &["row-1", "row-2", "row-3"],
            &["momentum-anchor"],
            3,
        )];
        let risk = vec![source_bound_test_opinion(
            "risk",
            LearnedAgentObjectiveV0::DownsideRisk,
            "risk-result",
            &["row-2", "row-3", "row-4"],
            &["risk-anchor"],
            4,
        )];
        let mapping = map_source_bound_opinions_v1(&momentum, &risk).unwrap();
        assert_eq!(mapping.scope_pairs.len(), 1);
        assert_eq!(
            mapping.scope_pairs[0].raw_scope_alignment,
            SourceBoundRawScopeAlignmentV1::DifferentInformationCutoff
        );
        assert_eq!(
            mapping.scope_pairs[0].anchor_alignment,
            SourceBoundAnchorAlignmentV1::DisjointAnchors
        );
        assert_eq!(
            mapping.scope_pairs[0].comparability,
            SourceBoundScopeComparabilityV1::SourceBoundButScopesNotComparable
        );
        assert_eq!(
            mapping.mapping_status,
            SourceBoundMappingStatusV1::SourceBoundButScopesNotComparable
        );
    }

    #[test]
    fn source_bound_ledger_record_round_trips_after_atomic_rename() {
        let registration = SourceBoundOpinionProtocolRegistrationV1::pre_registered();
        let mut ledger = new_source_bound_shadow_ledger_v1(&registration, "legacy".into()).unwrap();
        let (opinion, seal) = source_bound_test_opinion(
            "momentum",
            LearnedAgentObjectiveV0::DirectionalMomentum,
            "stored-result",
            &["row-1"],
            &["anchor-1"],
            1,
        );
        append_source_bound_opinion_v1(&mut ledger, opinion, seal).unwrap();
        let path = std::env::temp_dir().join(format!(
            "soma-source-bound-ledger-{}.json",
            ledger.ledger_digest_v1
        ));
        let _ = std::fs::remove_file(&path);
        write_source_bound_shadow_ledger_record_v1(&path, &ledger).unwrap();
        assert_eq!(
            read_source_bound_shadow_ledger_record_v1(&path).unwrap(),
            source_bound_shadow_ledger_record_v1(&ledger).unwrap()
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn v2_registration_reuses_the_exact_v1_scope_identities() {
        let snapshot = joint_snapshot(304);
        let campaign = MomentumLearningCampaignConfigV0::default();
        let v1 = joint_canonical_scope_registration_v1(&snapshot, &campaign).unwrap();
        let (_, v1_scopes) = issue_joint_canonical_scopes_v1(&snapshot, &v1).unwrap();
        let v2 = joint_canonical_scope_registration_v2(&snapshot, &campaign).unwrap();
        let v2_scopes =
            validate_joint_canonical_scope_registration_v2(&snapshot, &campaign, &v2).unwrap();
        assert_eq!(v2.parent_registration_digest_v1, v1.registration_digest_v1);
        assert_eq!(v2_scopes, v1_scopes);
        assert!(v2.scope_ranges_unchanged);
        assert!(v2.scope_selection_unchanged);
        assert!(v2.participant_configs_unchanged);
        assert!(v2.result_dependent_changes_forbidden);
    }

    #[test]
    fn v2_derived_snapshot_preserves_coupled_metadata_and_exact_child_policy() {
        let snapshot = joint_snapshot(304);
        let campaign = MomentumLearningCampaignConfigV0::default();
        let registration = joint_canonical_scope_registration_v1(&snapshot, &campaign).unwrap();
        let (_, scopes) = issue_joint_canonical_scopes_v1(&snapshot, &registration).unwrap();
        let derived = derive_joint_scope_snapshot_v2(&snapshot, &scopes[0]).unwrap();
        let repeated = derive_joint_scope_snapshot_v2(&snapshot, &scopes[0]).unwrap();
        assert_eq!(derived, repeated);
        assert_ne!(derived.derived_snapshot.snapshot_id, snapshot.snapshot_id);
        assert_eq!(
            derived.derived_snapshot.snapshot_id,
            snapshot_id_from_semantic_digest_v1(&derived.derived_snapshot.content_digest)
        );
        assert_eq!(
            derived.derived_snapshot.quality_summary.row_count,
            derived.derived_snapshot.row_count
        );
        assert!(derived.derivation_proof.all_invariants_pass);
        let policy = joint_scope_derived_evidence_policy_v2(&derived).unwrap();
        assert!(policy.exact_child_authorized);
        assert!(!policy.wildcard_authorization);
        let (inventory, pack) = joint_scope_momentum_pack_v2(&derived, &policy).unwrap();
        assert_eq!(inventory.accepted_series.len(), 1);
        assert_eq!(pack.series.len(), 1);
        let mut unverified = derived.clone();
        unverified.derived_snapshot.snapshot_id = "arbitrary-child".into();
        assert!(joint_scope_momentum_pack_v2(&unverified, &policy).is_err());
    }

    #[test]
    fn v2_ledger_rejects_nondeterministic_order() {
        let mut ledger = JointScopeReplayLedgerV2 {
            ledger_version: "joint-scope-replay-ledger-v2".into(),
            registration_digest_v2: "registration".into(),
            participant_result_digests: vec!["a".into(), "b".into()],
            deliberation_transcript_digests: vec![],
            ledger_digest_v2: String::new(),
        };
        ledger.ledger_digest_v2 = joint_v2_digest(&[
            ledger.ledger_version.clone(),
            ledger.registration_digest_v2.clone(),
            ledger.participant_result_digests.join(":"),
            ledger.deliberation_transcript_digests.join(":"),
        ]);
        validate_joint_scope_replay_ledger_v2(&ledger).unwrap();
        ledger.participant_result_digests.reverse();
        assert!(validate_joint_scope_replay_ledger_v2(&ledger).is_err());
    }

    #[test]
    fn v2_forensics_and_replay_keep_technical_health_separate_from_model_outcome() {
        let snapshot = joint_snapshot(304);
        let campaign = MomentumLearningCampaignConfigV0::default();
        let parent = joint_canonical_scope_registration_v1(&snapshot, &campaign).unwrap();
        let (_, scopes) = issue_joint_canonical_scopes_v1(&snapshot, &parent).unwrap();
        let first = forensic_joint_momentum_scope_v2(&snapshot, &scopes[0], &campaign).unwrap();
        let second = forensic_joint_momentum_scope_v2(&snapshot, &scopes[0], &campaign).unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first.root_cause,
            JointMomentumRootCauseV2::DerivedSnapshotIdentityMismatch
        );
        assert_eq!(
            first.execution_trace.first_failed_stage,
            Some(JointParticipantExecutionStageV2::DerivedSnapshotIdentity)
        );
        let registration = joint_canonical_scope_registration_v2(&snapshot, &campaign).unwrap();
        let replay =
            replay_joint_scope_results_v2(&snapshot, &scopes[0], &registration, &campaign).unwrap();
        assert_ne!(replay.derived_snapshot_id, snapshot.snapshot_id);
        assert!(!replay.momentum.execution_trace.trace_digest_v2.is_empty());
        assert!(!replay.risk.execution_trace.trace_digest_v2.is_empty());
        if replay.momentum.execution_trace.execution_health
            != JointParticipantExecutionHealthV2::Completed
        {
            assert_eq!(
                replay.momentum.execution_trace.model_evidence_outcome,
                JointParticipantModelEvidenceOutcomeV2::NotEvaluatedTechnicalFailure
            );
            assert_eq!(
                replay.momentum.execution_trace.operational_shadow_result,
                JointParticipantOperationalShadowResultV2::ShadowAbstainTechnicalFailure
            );
            assert!(replay.momentum.opinion_id.is_none());
        }
    }

    #[test]
    fn v2_aggregate_records_two_actionless_rounds_for_completed_abstentions() {
        let snapshot = joint_snapshot(304);
        let campaign = MomentumLearningCampaignConfigV0::default();
        let registration = joint_canonical_scope_registration_v2(&snapshot, &campaign).unwrap();
        let scopes =
            validate_joint_canonical_scope_registration_v2(&snapshot, &campaign, &registration)
                .unwrap();
        let results = scopes
            .iter()
            .enumerate()
            .map(|(index, scope)| {
                let momentum_pair = source_bound_test_opinion(
                    "momentum",
                    LearnedAgentObjectiveV0::DirectionalMomentum,
                    &format!("momentum-{index}"),
                    &["row-1", "row-2"],
                    &["momentum-anchor"],
                    scope.information_cutoff_timestamp,
                );
                let risk_pair = source_bound_test_opinion(
                    "risk",
                    LearnedAgentObjectiveV0::DownsideRisk,
                    &format!("risk-{index}"),
                    &["row-1", "row-2"],
                    &["risk-anchor"],
                    scope.information_cutoff_timestamp,
                );
                let mut momentum_trace = new_execution_trace_v2(
                    scope,
                    "momentum".into(),
                    LearnedAgentObjectiveV0::DirectionalMomentum,
                );
                momentum_trace.execution_health = JointParticipantExecutionHealthV2::Completed;
                momentum_trace.model_evidence_outcome =
                    JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal;
                momentum_trace.operational_shadow_result =
                    JointParticipantOperationalShadowResultV2::ShadowAbstainNoSignal;
                finish_execution_trace_v2(&mut momentum_trace);
                let mut risk_trace = new_execution_trace_v2(
                    scope,
                    "risk".into(),
                    LearnedAgentObjectiveV0::DownsideRisk,
                );
                risk_trace.execution_health = JointParticipantExecutionHealthV2::Completed;
                risk_trace.model_evidence_outcome =
                    JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal;
                risk_trace.operational_shadow_result =
                    JointParticipantOperationalShadowResultV2::ShadowAbstainNoSignal;
                finish_execution_trace_v2(&mut risk_trace);
                let mut momentum = JointScopeParticipantReplayResultV2 {
                    joint_scope_id: scope.joint_scope_id.clone(),
                    joint_scope_digest: scope.scope_digest_v1.clone(),
                    participant_agent_id: "momentum".into(),
                    objective: LearnedAgentObjectiveV0::DirectionalMomentum,
                    execution_trace: momentum_trace,
                    completed_result_digest: Some(format!("momentum-{index}")),
                    anchor_scope_digest: Some("momentum-anchor-scope".into()),
                    anchor_status: JointAnchorAuditStatusV2::CompleteWithoutSelectedCheckpoint,
                    opinion_id: Some(momentum_pair.0.opinion_id.clone()),
                    seal_digest: Some(momentum_pair.1.seal_digest_v1.clone()),
                    sealed_opinion: Some(momentum_pair),
                    result_digest_v2: String::new(),
                };
                momentum.result_digest_v2 = participant_result_digest_v2(&momentum);
                let mut risk = JointScopeParticipantReplayResultV2 {
                    joint_scope_id: scope.joint_scope_id.clone(),
                    joint_scope_digest: scope.scope_digest_v1.clone(),
                    participant_agent_id: "risk".into(),
                    objective: LearnedAgentObjectiveV0::DownsideRisk,
                    execution_trace: risk_trace,
                    completed_result_digest: Some(format!("risk-{index}")),
                    anchor_scope_digest: Some("risk-anchor-scope".into()),
                    anchor_status: JointAnchorAuditStatusV2::Complete,
                    opinion_id: Some(risk_pair.0.opinion_id.clone()),
                    seal_digest: Some(risk_pair.1.seal_digest_v1.clone()),
                    sealed_opinion: Some(risk_pair),
                    result_digest_v2: String::new(),
                };
                risk.result_digest_v2 = participant_result_digest_v2(&risk);
                JointScopeReplayResultV2 {
                    replay_version: "joint-canonical-scope-replay-v2".into(),
                    registration_digest_v2: registration.registration_digest_v2.clone(),
                    joint_scope_id: scope.joint_scope_id.clone(),
                    joint_scope_digest: scope.scope_digest_v1.clone(),
                    derived_snapshot_id: format!("child-{index}"),
                    derivation_digest_v2: format!("derivation-{index}"),
                    evidence_policy_digest_v2: format!("policy-{index}"),
                    momentum,
                    risk,
                    pair_eligible: true,
                    result_digest_v2: format!("replay-{index}"),
                }
            })
            .collect::<Vec<_>>();
        let (aggregate, ledger) =
            aggregate_joint_scope_replays_v2(&registration, &results).unwrap();
        assert!(aggregate.full_aggregate_composed);
        assert_eq!(aggregate.completed_pair_count, 2);
        assert_eq!(
            aggregate.relationships,
            vec![JointScopeRelationshipV2::BothAbstained; 2]
        );
        assert!(aggregate.deliberations.iter().all(|value| {
            value.round_count == 2
                && !value.chair_observed
                && !value.vote_created
                && !value.reward_created
                && !value.penalty_created
                && !value.execution_created
        }));
        validate_joint_scope_replay_ledger_v2(&ledger).unwrap();
    }

    #[test]
    fn v3_scope0_closure_audit_is_deterministic_and_preserves_preclosure_values() {
        let (result, regime, closed) = v3_audit_fixture();
        let first = v3_audit(&result, &regime, &closed);
        let second = v3_audit(&result, &regime, &closed);
        assert_eq!(first, second);
        assert!(first.all_invariants_pass);
        assert_eq!(
            first.failure_class,
            MomentumClosureFailureClassV3::NoFailure
        );
        assert!(first.validator_error.is_none());
        assert_eq!(
            first.preclosure.campaign_report_digest,
            result.report_digest
        );
        assert_eq!(
            first.preclosure.no_signal_window_count,
            result.no_signal_windows
        );
        assert_eq!(
            first.preclosure.selected_checkpoint_count,
            result.selected_checkpoint_windows
        );
    }

    #[test]
    fn v3_closure_audit_reports_field_digest_and_outcome_mutations() {
        let (result, regime, closed) = v3_audit_fixture();
        let mut changed_regime = closed.clone();
        changed_regime.regime.row_count += 1;
        assert_eq!(
            v3_audit(&result, &regime, &changed_regime).first_failed_invariant,
            Some(MomentumClosureInvariantV3::RegimeRowCount)
        );
        let mut changed_digest = closed.clone();
        changed_digest.report_digest.push('x');
        assert_eq!(
            v3_audit(&result, &regime, &changed_digest).first_failed_invariant,
            Some(MomentumClosureInvariantV3::ReportDigest)
        );
        let mut changed_outcome = closed;
        changed_outcome.model_evidence_outcome = RegimeModelEvidenceOutcomeV0::InsufficientEvidence;
        assert_eq!(
            v3_audit(&result, &regime, &changed_outcome).first_failed_invariant,
            Some(MomentumClosureInvariantV3::ModelEvidenceOutcome)
        );
    }

    #[test]
    fn v3_closure_audit_preserves_typed_validator_error_for_invalid_closure() {
        let (result, regime, mut closed) = v3_audit_fixture();
        closed.selected_checkpoint_windows = closed.campaign_window_count + 1;
        let audit = v3_audit(&result, &regime, &closed);
        assert_eq!(
            audit.first_failed_invariant,
            Some(MomentumClosureInvariantV3::SelectedCheckpointRange)
        );
        assert_eq!(
            audit.validator_error.as_deref(),
            Some("MissingRequiredMetric")
        );
        assert_eq!(
            audit.failure_class,
            MomentumClosureFailureClassV3::MultipleMismatches
        );
    }

    #[test]
    fn v3_registration_binds_parent_scopes_configs_and_preclosure_freeze() {
        let snapshot = joint_snapshot(304);
        let campaign = MomentumLearningCampaignConfigV0::default();
        let parent = joint_canonical_scope_registration_v2(&snapshot, &campaign).unwrap();
        let scopes =
            validate_joint_canonical_scope_registration_v2(&snapshot, &campaign, &parent).unwrap();
        let audits = scopes
            .iter()
            .enumerate()
            .map(|(index, scope)| MomentumClosedResultContractAuditV3 {
                audit_version: "test".into(),
                joint_scope_id: scope.joint_scope_id.clone(),
                open_result_digest: format!("open-{index}"),
                closed_result_digest: format!("closed-{index}"),
                regime_reference_digest: format!("regime-{index}"),
                preclosure: MomentumPreClosureEvidenceV3 {
                    campaign_report_digest: format!("report-{index}"),
                    campaign_window_count: 2,
                    final_verdict: "test".into(),
                    no_signal_window_count: if index == 1 { 1 } else { 0 },
                    selected_checkpoint_count: 1,
                    support_counts: vec![1, 0, 0, 1, 0, 0, 0],
                    encoder_digest: format!("encoder-{index}"),
                    pack_digest: format!("pack-{index}"),
                    derived_snapshot_digest: format!("derived-{index}"),
                    preclosure_digest_v3: format!("preclosure-{index}"),
                },
                invariant_results: vec![],
                first_failed_invariant: None,
                validator_error: None,
                failure_class: MomentumClosureFailureClassV3::NoFailure,
                all_invariants_pass: true,
                audit_digest_v3: format!("audit-{index}"),
            })
            .collect::<Vec<_>>();
        let registration =
            joint_canonical_scope_registration_v3(&snapshot, &campaign, &audits).unwrap();
        assert_eq!(
            registration.parent_registration_digest_v2,
            parent.registration_digest_v2
        );
        assert_eq!(
            registration.correction_failure_class,
            MomentumClosureFailureClassV3::StaleValidatorContract
        );
        assert!(registration.scope_ranges_unchanged);
        assert!(registration.participant_configs_unchanged);
        assert!(registration.preclosure_results_unchanged);
        assert!(registration.scope0_non_regression_required);
        assert!(registration.result_dependent_model_changes_forbidden);
        assert_eq!(
            validate_joint_canonical_scope_registration_v3(&snapshot, &campaign, &registration)
                .unwrap(),
            scopes
        );
    }

    fn chair_shadow_test_participant(
        scope: &JointCanonicalHistoricalScopeV1,
        index: usize,
        agent_id: &str,
        objective: LearnedAgentObjectiveV0,
        operational_result: JointParticipantOperationalShadowResultV2,
    ) -> JointScopeParticipantReplayResultV2 {
        let result_digest = format!("{agent_id}-chair-{index}");
        let pair = source_bound_test_opinion(
            agent_id,
            objective,
            &result_digest,
            &["row-1", "row-2"],
            &["anchor-1"],
            scope.information_cutoff_timestamp,
        );
        let mut trace = new_execution_trace_v2(scope, agent_id.into(), objective);
        trace.execution_health = JointParticipantExecutionHealthV2::Completed;
        trace.model_evidence_outcome = if matches!(
            operational_result,
            JointParticipantOperationalShadowResultV2::ShadowPredictionResearchOnly
        ) {
            JointParticipantModelEvidenceOutcomeV2::UsableValidationSignal
        } else {
            JointParticipantModelEvidenceOutcomeV2::NoUsableValidationSignal
        };
        trace.operational_shadow_result = operational_result;
        finish_execution_trace_v2(&mut trace);
        let mut participant = JointScopeParticipantReplayResultV2 {
            joint_scope_id: scope.joint_scope_id.clone(),
            joint_scope_digest: scope.scope_digest_v1.clone(),
            participant_agent_id: agent_id.into(),
            objective,
            execution_trace: trace,
            completed_result_digest: Some(result_digest),
            anchor_scope_digest: Some(format!("{agent_id}-anchor-scope")),
            anchor_status: JointAnchorAuditStatusV2::Complete,
            opinion_id: Some(pair.0.opinion_id.clone()),
            seal_digest: Some(pair.1.seal_digest_v1.clone()),
            sealed_opinion: Some(pair),
            result_digest_v2: String::new(),
        };
        participant.result_digest_v2 = participant_result_digest_v2(&participant);
        participant
    }

    fn chair_shadow_test_evidence() -> ChairShadowObservationEvidenceV0 {
        let snapshot = joint_snapshot(304);
        let campaign = MomentumLearningCampaignConfigV0::default();
        let parent = joint_canonical_scope_registration_v2(&snapshot, &campaign).unwrap();
        let scopes =
            validate_joint_canonical_scope_registration_v2(&snapshot, &campaign, &parent).unwrap();
        let audits = scopes
            .iter()
            .enumerate()
            .map(|(index, scope)| MomentumClosedResultContractAuditV3 {
                audit_version: "chair-shadow-test-audit-v3".into(),
                joint_scope_id: scope.joint_scope_id.clone(),
                open_result_digest: format!("open-{index}"),
                closed_result_digest: format!("closed-{index}"),
                regime_reference_digest: format!("regime-{index}"),
                preclosure: MomentumPreClosureEvidenceV3 {
                    campaign_report_digest: format!("report-{index}"),
                    campaign_window_count: 2,
                    final_verdict: "test".into(),
                    no_signal_window_count: if index == 1 { 1 } else { 0 },
                    selected_checkpoint_count: 1,
                    support_counts: vec![1, 0, 0, 1, 0, 0, 0],
                    encoder_digest: format!("encoder-{index}"),
                    pack_digest: format!("pack-{index}"),
                    derived_snapshot_digest: format!("derived-{index}"),
                    preclosure_digest_v3: format!("preclosure-{index}"),
                },
                invariant_results: vec![],
                first_failed_invariant: None,
                validator_error: None,
                failure_class: MomentumClosureFailureClassV3::NoFailure,
                all_invariants_pass: true,
                audit_digest_v3: format!("audit-{index}"),
            })
            .collect::<Vec<_>>();
        let registration =
            joint_canonical_scope_registration_v3(&snapshot, &campaign, &audits).unwrap();
        let results = scopes
            .iter()
            .enumerate()
            .map(|(index, scope)| {
                let momentum = chair_shadow_test_participant(
                    scope,
                    index,
                    "momentum",
                    LearnedAgentObjectiveV0::DirectionalMomentum,
                    JointParticipantOperationalShadowResultV2::ShadowAbstainNoSignal,
                );
                let risk = chair_shadow_test_participant(
                    scope,
                    index,
                    "risk",
                    LearnedAgentObjectiveV0::DownsideRisk,
                    if index == 0 {
                        JointParticipantOperationalShadowResultV2::ShadowAbstainNoSignal
                    } else {
                        JointParticipantOperationalShadowResultV2::ShadowPredictionResearchOnly
                    },
                );
                let replay_result_v2 = JointScopeReplayResultV2 {
                    replay_version: "joint-canonical-scope-replay-v2".into(),
                    registration_digest_v2: parent.registration_digest_v2.clone(),
                    joint_scope_id: scope.joint_scope_id.clone(),
                    joint_scope_digest: scope.scope_digest_v1.clone(),
                    derived_snapshot_id: format!("child-{index}"),
                    derivation_digest_v2: format!("derivation-{index}"),
                    evidence_policy_digest_v2: format!("policy-{index}"),
                    momentum,
                    risk,
                    pair_eligible: true,
                    result_digest_v2: format!("replay-{index}"),
                };
                JointScopeReplayResultV3 {
                    replay_version: "joint-canonical-scope-replay-v3".into(),
                    registration_digest_v3: registration.registration_digest_v3.clone(),
                    joint_scope_id: scope.joint_scope_id.clone(),
                    joint_scope_digest: scope.scope_digest_v1.clone(),
                    preclosure_digest_v3: registration.preclosure_result_digests[index].clone(),
                    closure_audit_digest_v3: audits[index].audit_digest_v3.clone(),
                    parent_result_digest_v2: replay_result_v2.result_digest_v2.clone(),
                    replay_result_v2,
                    result_digest_v3: format!("chair-shadow-result-{index}"),
                }
            })
            .collect::<Vec<_>>();
        let (aggregate, ledger) =
            aggregate_joint_scope_replays_v3(&registration, &results).unwrap();
        chair_shadow_observation_evidence_v0(registration, results, aggregate, ledger).unwrap()
    }

    pub(super) fn chair_shadow_test_observation_report_for_owner_review_v0()
    -> ChairShadowObservationReportV0 {
        observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap()
    }

    fn chair_shadow_test_packet() -> (
        ChairShadowObservationEvidenceV0,
        ChairShadowObservationPacketV0,
    ) {
        let evidence = chair_shadow_test_evidence();
        let packet = chair_shadow_observation_packet_v0(&evidence).unwrap();
        (evidence, packet)
    }

    #[test]
    fn chair_shadow_packet_is_deterministic_and_retrospective_only() {
        let evidence = chair_shadow_test_evidence();
        assert_eq!(
            chair_shadow_observation_packet_v0(&evidence).unwrap(),
            chair_shadow_observation_packet_v0(&evidence).unwrap()
        );
    }

    #[test]
    fn chair_shadow_intake_accepts_verified_v3_source() {
        let (evidence, packet) = chair_shadow_test_packet();
        let mut inbox = new_chair_shadow_observation_inbox_v0();
        assert_eq!(
            intake_chair_shadow_observation_packet_v0(&mut inbox, &packet, &evidence).status,
            ChairObservationReceiptStatusV0::AcceptedRetrospectiveObservationOnly
        );
    }

    #[test]
    fn chair_shadow_rejects_altered_registration() {
        let (mut evidence, packet) = chair_shadow_test_packet();
        evidence.registration.registration_digest_v3.push('x');
        assert_eq!(
            intake_chair_shadow_observation_packet_v0(
                &mut new_chair_shadow_observation_inbox_v0(),
                &packet,
                &evidence
            )
            .status,
            ChairObservationReceiptStatusV0::InvalidRegistration
        );
    }

    #[test]
    fn chair_shadow_rejects_altered_ledger() {
        let (mut evidence, packet) = chair_shadow_test_packet();
        evidence.ledger.ledger_digest_v3.push('x');
        assert_eq!(
            intake_chair_shadow_observation_packet_v0(
                &mut new_chair_shadow_observation_inbox_v0(),
                &packet,
                &evidence
            )
            .status,
            ChairObservationReceiptStatusV0::InvalidLedger
        );
    }

    #[test]
    fn chair_shadow_rejects_altered_aggregate() {
        let (mut evidence, packet) = chair_shadow_test_packet();
        evidence.aggregate.aggregate_digest_v3.push('x');
        assert_eq!(
            intake_chair_shadow_observation_packet_v0(
                &mut new_chair_shadow_observation_inbox_v0(),
                &packet,
                &evidence
            )
            .status,
            ChairObservationReceiptStatusV0::InvalidAggregate
        );
    }

    #[test]
    fn chair_shadow_rejects_invalid_opinion_seal_reference() {
        let (evidence, mut packet) = chair_shadow_test_packet();
        packet.opinion_seal_digests[0].push('x');
        packet.packet_digest = chair_packet_digest_v0(&packet);
        assert_eq!(
            intake_chair_shadow_observation_packet_v0(
                &mut new_chair_shadow_observation_inbox_v0(),
                &packet,
                &evidence
            )
            .status,
            ChairObservationReceiptStatusV0::InvalidOpinionSeal
        );
    }

    #[test]
    fn chair_shadow_rejects_invalid_transcript_reference() {
        let (evidence, mut packet) = chair_shadow_test_packet();
        packet.transcript_digests[0].push('x');
        packet.packet_digest = chair_packet_digest_v0(&packet);
        assert_eq!(
            intake_chair_shadow_observation_packet_v0(
                &mut new_chair_shadow_observation_inbox_v0(),
                &packet,
                &evidence
            )
            .status,
            ChairObservationReceiptStatusV0::InvalidTranscript
        );
    }

    #[test]
    fn chair_shadow_rejects_prospective_claims() {
        let (evidence, mut packet) = chair_shadow_test_packet();
        packet.prospective = true;
        packet.packet_digest = chair_packet_digest_v0(&packet);
        assert_eq!(
            intake_chair_shadow_observation_packet_v0(
                &mut new_chair_shadow_observation_inbox_v0(),
                &packet,
                &evidence
            )
            .status,
            ChairObservationReceiptStatusV0::ProspectiveClaimForbidden
        );
    }

    #[test]
    fn chair_shadow_rejects_decision_authority() {
        let (evidence, mut packet) = chair_shadow_test_packet();
        packet.authority.vote_allowed = true;
        packet.packet_digest = chair_packet_digest_v0(&packet);
        assert_eq!(
            intake_chair_shadow_observation_packet_v0(
                &mut new_chair_shadow_observation_inbox_v0(),
                &packet,
                &evidence
            )
            .status,
            ChairObservationReceiptStatusV0::AuthorityViolation
        );
    }

    #[test]
    fn chair_shadow_duplicate_packet_fails_closed() {
        let (evidence, packet) = chair_shadow_test_packet();
        let mut inbox = new_chair_shadow_observation_inbox_v0();
        intake_chair_shadow_observation_packet_v0(&mut inbox, &packet, &evidence);
        assert_eq!(
            intake_chair_shadow_observation_packet_v0(&mut inbox, &packet, &evidence).status,
            ChairObservationReceiptStatusV0::DuplicatePacket
        );
    }

    #[test]
    fn chair_shadow_report_has_no_chair_runtime_invocation() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        assert_eq!(report.inbox.chair_runtime_invocations, 0);
    }

    #[test]
    fn chair_shadow_report_has_no_chair_decision() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        assert_eq!(report.inbox.chair_decisions_created, 0);
    }

    #[test]
    fn chair_shadow_report_has_no_votes() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        assert_eq!(report.inbox.votes_created, 0);
    }

    #[test]
    fn chair_shadow_report_has_no_rewards_or_penalties() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        assert_eq!(
            (report.inbox.rewards_created, report.inbox.penalties_created),
            (0, 0)
        );
    }

    #[test]
    fn chair_shadow_report_has_no_speaking_right_changes() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        assert_eq!(report.inbox.speaking_right_changes, 0);
    }

    #[test]
    fn chair_shadow_report_has_no_risk_handoff_or_execution() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        assert_eq!(
            (report.inbox.risk_handoffs, report.inbox.executions_created),
            (0, 0)
        );
    }

    #[test]
    fn chair_shadow_firewall_proof_is_complete() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        assert!(report.firewall_proof.all_invariants_pass);
        assert!(report.firewall_proof.packet_cannot_become_vote);
        assert!(report.firewall_proof.packet_cannot_become_chair_input);
    }

    #[test]
    fn chair_shadow_receipt_reports_two_scopes_four_opinions_and_three_abstentions() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        assert_eq!(report.receipt.observed_scope_count, 2);
        assert_eq!(report.receipt.observed_opinion_count, 4);
        assert_eq!(report.receipt.observed_abstention_count, 3);
    }

    #[test]
    fn chair_shadow_receipt_reports_relationship_categories_from_source() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        assert_eq!(
            report.receipt.relationship_summary,
            vec![
                ChairObservedRelationshipCountV0 {
                    category: ChairObservedRelationshipCategoryV0::BothAbstained,
                    count: 1,
                },
                ChairObservedRelationshipCountV0 {
                    category: ChairObservedRelationshipCategoryV0::MomentumAbstained,
                    count: 1,
                },
            ]
        );
    }

    #[test]
    fn chair_shadow_receipt_is_deterministic() {
        let evidence = chair_shadow_test_evidence();
        assert_eq!(
            observe_chair_shadow_observation_v0(&evidence).unwrap(),
            observe_chair_shadow_observation_v0(&evidence).unwrap()
        );
    }

    #[test]
    fn chair_shadow_receipt_contains_only_sanitized_wire_fields() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        let encoded = serde_json::to_string(&report).unwrap();
        assert!(!encoded.contains("\"probability\":"));
        assert!(!encoded.contains("\"council_score\":"));
        assert!(!encoded.contains("\"size_multiplier\":"));
    }

    #[test]
    fn chair_shadow_storage_reopens_and_validates() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        let path = std::env::temp_dir().join(format!(
            "chair-shadow-observation-{}-reopen.json",
            std::process::id()
        ));
        let stored = append_chair_shadow_observation_storage_v0(&path, &report).unwrap();
        assert_eq!(
            read_chair_shadow_observation_storage_v0(&path).unwrap(),
            stored
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn chair_shadow_storage_digest_is_path_independent() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        let base = std::env::temp_dir();
        let left = base.join(format!(
            "chair-shadow-observation-{}-left.json",
            std::process::id()
        ));
        let right = base.join(format!(
            "chair-shadow-observation-{}-right.json",
            std::process::id()
        ));
        let left_storage = append_chair_shadow_observation_storage_v0(&left, &report).unwrap();
        let right_storage = append_chair_shadow_observation_storage_v0(&right, &report).unwrap();
        assert_eq!(left_storage.storage_digest, right_storage.storage_digest);
        std::fs::remove_file(left).unwrap();
        std::fs::remove_file(right).unwrap();
    }

    #[test]
    fn chair_shadow_storage_is_idempotent_for_the_same_packet() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        let path = std::env::temp_dir().join(format!(
            "chair-shadow-observation-{}-idempotent.json",
            std::process::id()
        ));
        let first = append_chair_shadow_observation_storage_v0(&path, &report).unwrap();
        let second = append_chair_shadow_observation_storage_v0(&path, &report).unwrap();
        assert_eq!(first, second);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn chair_shadow_storage_rejects_tampered_digest() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        let path = std::env::temp_dir().join(format!(
            "chair-shadow-observation-{}-tamper.json",
            std::process::id()
        ));
        append_chair_shadow_observation_storage_v0(&path, &report).unwrap();
        let mut storage = read_chair_shadow_observation_storage_v0(&path).unwrap();
        storage.storage_digest.push('x');
        assert!(validate_chair_shadow_observation_storage_v0(&storage).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn chair_shadow_active_committee_count_is_fixed_and_authority_is_disabled() {
        let report = observe_chair_shadow_observation_v0(&chair_shadow_test_evidence()).unwrap();
        assert_eq!(report.active_committee_count, 3);
        assert!(report.packet.authority.advisory_only);
        assert!(!report.packet.authority.execution_allowed);
    }

    fn prospective_maturity_event(
        objective: LearnedAgentObjectiveV0,
        prediction_timestamp: u64,
        horizon_rows: usize,
    ) -> LearnedProspectiveEventV0 {
        let agent_id = match objective {
            LearnedAgentObjectiveV0::DirectionalMomentum => MOMENTUM_AGENT_ID_V0,
            LearnedAgentObjectiveV0::DownsideRisk => CYCLE_RISK_SHADOW_AGENT_ID_V0,
        };
        let mut event = LearnedProspectiveEventV0 {
            event_version: "learned-prospective-event-v0".into(),
            event_id: format!("event-{agent_id}-{horizon_rows}"),
            agent_id: agent_id.into(),
            objective,
            challenge_digest: format!("challenge-{agent_id}"),
            shared_raw_evidence_digest: "shared-evidence".into(),
            frozen_model_artifact_digests: vec![format!("artifact-{agent_id}")],
            input_digest: "shared-evidence".into(),
            support_status_digest: "support".into(),
            operational_outcome:
                ProspectiveOperationalOutcomeV0::ShadowAbstentionSupportUnavailable,
            abstention_reason: Some("frozen_support_unavailable".into()),
            prediction_timestamp,
            maturity_timestamp: prediction_timestamp
                + horizon_rows as u64 * PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0,
            horizon_digest: stable_hash_string(&format!(
                "external-prospective-horizon-v0:{objective:?}:{horizon_rows}"
            )),
            probability_bits_sealed: true,
            label_accessed: false,
            event_digest: String::new(),
        };
        event.event_digest = learned_prospective_event_digest_v0(&event);
        event
    }

    fn prospective_maturity_fixture() -> (
        LearnedProspectiveEventV0,
        LearnedProspectiveEventV0,
        Vec<ProspectiveEventMaturityPlanV0>,
        ProspectiveOneTimeOpeningRegistrationV0,
    ) {
        let momentum = prospective_maturity_event(
            LearnedAgentObjectiveV0::DirectionalMomentum,
            1_800_000_000_000,
            1,
        );
        let risk =
            prospective_maturity_event(LearnedAgentObjectiveV0::DownsideRisk, 1_800_000_000_000, 4);
        let (registration, plans) = pre_register_prospective_one_time_opening_v0(
            &momentum,
            &risk,
            1,
            4,
            "momentum-label",
            "risk-label",
            "credential-free-public-btc-daily",
            "finalized-contiguous-utc-daily",
            vec!["momentum-metric".into(), "risk-metric".into()],
        )
        .unwrap();
        (momentum, risk, plans, registration)
    }

    #[test]
    fn prospective_maturity_plans_are_objective_specific_and_horizon_derived() {
        let (_, _, plans, registration) = prospective_maturity_fixture();
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].required_finalized_row_count, 1);
        assert_eq!(plans[1].required_finalized_row_count, 4);
        assert_ne!(plans[0].horizon_digest, plans[1].horizon_digest);
        assert_eq!(registration.maximum_response_rows, 4);
        assert!(
            validate_prospective_one_time_opening_registration_v0(&registration, &plans).is_ok()
        );
    }

    #[test]
    fn prospective_maturity_plan_rejects_wrong_horizon_and_event_mutation() {
        let (momentum, _, _, _) = prospective_maturity_fixture();
        assert!(
            derive_prospective_event_maturity_plan_v0(&momentum, 2, "momentum-label", "source")
                .is_err()
        );
        let mut changed = momentum;
        changed.maturity_timestamp += PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0;
        assert!(
            derive_prospective_event_maturity_plan_v0(&changed, 1, "momentum-label", "source")
                .is_err()
        );
    }

    #[test]
    fn prospective_maturity_registration_rejects_excessive_union() {
        let momentum = prospective_maturity_event(
            LearnedAgentObjectiveV0::DirectionalMomentum,
            1_800_000_000_000,
            1,
        );
        let risk = prospective_maturity_event(
            LearnedAgentObjectiveV0::DownsideRisk,
            1_800_000_000_000,
            MAXIMUM_PROSPECTIVE_OUTCOME_RESPONSE_ROWS_V0 + 1,
        );
        assert!(
            pre_register_prospective_one_time_opening_v0(
                &momentum,
                &risk,
                1,
                MAXIMUM_PROSPECTIVE_OUTCOME_RESPONSE_ROWS_V0 + 1,
                "momentum-label",
                "risk-label",
                "source",
                "finalization",
                vec!["momentum-metric".into(), "risk-metric".into()],
            )
            .is_err()
        );
    }

    #[test]
    fn prospective_maturity_evidence_requires_exact_finalized_union() {
        let (_, _, plans, _) = prospective_maturity_fixture();
        let empty = assess_prospective_outcome_evidence_v0(&plans, "KRW:BTC", &[]);
        assert_eq!(
            empty.status,
            ProspectiveOutcomeEvidenceStatusV0::NoOutcomeRows
        );
        let partial_rows = vec![ProspectiveOutcomeEvidenceRowV0 {
            series_id: "KRW:BTC".into(),
            timestamp: plans[0].required_outcome_start_timestamp,
            canonical_row_digest: "row-1".into(),
            finalized: true,
        }];
        let partial = assess_prospective_outcome_evidence_v0(&plans, "KRW:BTC", &partial_rows);
        assert_eq!(
            partial.status,
            ProspectiveOutcomeEvidenceStatusV0::PartialOutcomeRows
        );
        let missing_rows = vec![
            partial_rows[0].clone(),
            ProspectiveOutcomeEvidenceRowV0 {
                series_id: "KRW:BTC".into(),
                timestamp: plans[1].required_outcome_start_timestamp
                    + 2 * PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0,
                canonical_row_digest: "row-3".into(),
                finalized: true,
            },
        ];
        assert_eq!(
            assess_prospective_outcome_evidence_v0(&plans, "KRW:BTC", &missing_rows).status,
            ProspectiveOutcomeEvidenceStatusV0::MissingRequiredTimestamp
        );
        let mut non_finalized = partial_rows.clone();
        non_finalized[0].finalized = false;
        assert_eq!(
            assess_prospective_outcome_evidence_v0(&plans, "KRW:BTC", &non_finalized).status,
            ProspectiveOutcomeEvidenceStatusV0::NonFinalizedRow
        );
        let mut duplicate = partial_rows.clone();
        duplicate.push(partial_rows[0].clone());
        assert_eq!(
            assess_prospective_outcome_evidence_v0(&plans, "KRW:BTC", &duplicate).status,
            ProspectiveOutcomeEvidenceStatusV0::DuplicateRows
        );
        let mut complete = (0..plans[1].required_finalized_row_count)
            .map(|offset| ProspectiveOutcomeEvidenceRowV0 {
                series_id: "KRW:BTC".into(),
                timestamp: plans[1].required_outcome_start_timestamp
                    + offset as u64 * PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0,
                canonical_row_digest: format!("row-{offset}"),
                finalized: true,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            assess_prospective_outcome_evidence_v0(&plans, "KRW:BTC", &complete).status,
            ProspectiveOutcomeEvidenceStatusV0::CompleteVerified
        );
        complete.push(ProspectiveOutcomeEvidenceRowV0 {
            series_id: "KRW:BTC".into(),
            timestamp: plans[1].required_outcome_end_timestamp
                + PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0,
            canonical_row_digest: "extra".into(),
            finalized: true,
        });
        assert_eq!(
            assess_prospective_outcome_evidence_v0(&plans, "KRW:BTC", &complete).status,
            ProspectiveOutcomeEvidenceStatusV0::ChronologyInvalid
        );
    }

    #[test]
    fn prospective_maturity_readiness_stays_closed_without_time_and_authorization() {
        let (_, _, plans, registration) = prospective_maturity_fixture();
        let before = prospective_opening_readiness_v0(
            &plans[0],
            plans[0].maturity_timestamp - 1,
            ProspectiveOutcomeEvidenceStatusV0::CompleteVerified,
            true,
            true,
            0,
        );
        assert_eq!(before, ProspectiveOpeningReadinessV0::AwaitingTimeMaturity);
        let missing = prospective_opening_readiness_v0(
            &plans[0],
            plans[0].maturity_timestamp,
            ProspectiveOutcomeEvidenceStatusV0::NoOutcomeRows,
            true,
            true,
            0,
        );
        assert_eq!(
            missing,
            ProspectiveOpeningReadinessV0::TimeMatureOutcomeRowsMissing
        );
        let authorization = ProspectiveOpeningAuthorizationV0 {
            authorization_version: "prospective-opening-authorization-v0".into(),
            opening_registration_digest: registration.registration_digest.clone(),
            authorized_event_digests: vec![
                registration.momentum_event_digest.clone(),
                registration.risk_event_digest.clone(),
            ],
            authorized_outcome_evidence_digest: "evidence".into(),
            explicit_owner_authorization: true,
            one_time_only: true,
            label_open_count_before: 0,
            authorization_digest: String::new(),
        };
        let mut authorization = authorization;
        authorization.authorization_digest =
            prospective_opening_authorization_digest_v0(&authorization);
        assert!(
            validate_prospective_opening_authorization_v0(
                &authorization,
                &registration,
                missing,
                "evidence",
                0,
            )
            .is_err()
        );
        assert!(
            validate_prospective_opening_authorization_v0(
                &authorization,
                &registration,
                ProspectiveOpeningReadinessV0::ReadyForExplicitOpening,
                "evidence",
                1,
            )
            .is_err()
        );
    }

    #[test]
    fn prospective_outcome_request_readiness_uses_both_finalized_row_boundaries() {
        let (_, _, plans, registration) = prospective_maturity_fixture();
        let momentum_boundary =
            plans[0].required_outcome_end_timestamp + PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0;
        let risk_boundary =
            plans[1].required_outcome_end_timestamp + PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0;
        assert_eq!(
            prospective_outcome_request_readiness_v0(
                &registration,
                &plans,
                momentum_boundary - 1,
                true,
                false,
                false,
            ),
            ProspectiveOutcomeRequestReadinessV0::AwaitingBothTimeMaturities
        );
        assert_eq!(
            prospective_outcome_request_readiness_v0(
                &registration,
                &plans,
                momentum_boundary,
                true,
                false,
                false,
            ),
            ProspectiveOutcomeRequestReadinessV0::AwaitingRiskTimeMaturity
        );
        assert_eq!(
            prospective_outcome_request_readiness_v0(
                &registration,
                &plans,
                risk_boundary,
                true,
                false,
                false,
            ),
            ProspectiveOutcomeRequestReadinessV0::ReadyForExplicitRequest
        );
    }

    #[test]
    fn prospective_outcome_request_readiness_fails_closed_on_integrity_and_budget_state() {
        let (_, _, plans, registration) = prospective_maturity_fixture();
        let ready_at =
            plans[1].required_outcome_end_timestamp + PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0;
        assert_eq!(
            prospective_outcome_request_readiness_v0(
                &registration,
                &plans,
                ready_at,
                false,
                false,
                false,
            ),
            ProspectiveOutcomeRequestReadinessV0::EventIntegrityInvalid
        );
        let mut changed_plan = plans.clone();
        changed_plan[1].required_outcome_end_timestamp += PROSPECTIVE_OUTCOME_ROW_INTERVAL_MS_V0;
        assert_eq!(
            prospective_outcome_request_readiness_v0(
                &registration,
                &changed_plan,
                ready_at,
                true,
                false,
                false,
            ),
            ProspectiveOutcomeRequestReadinessV0::RegistrationInvalid
        );
        assert_eq!(
            prospective_outcome_request_readiness_v0(
                &registration,
                &plans,
                ready_at,
                true,
                true,
                false,
            ),
            ProspectiveOutcomeRequestReadinessV0::RequestAlreadyAttempted
        );
        assert_eq!(
            prospective_outcome_request_readiness_v0(
                &registration,
                &plans,
                ready_at,
                true,
                true,
                true,
            ),
            ProspectiveOutcomeRequestReadinessV0::OutcomeEvidenceAlreadyPresent
        );
    }

    #[test]
    fn prospective_maturity_preflight_values_are_deterministic_and_authority_free() {
        let (_, _, plans, registration) = prospective_maturity_fixture();
        let first = assess_prospective_outcome_evidence_v0(&plans, "KRW:BTC", &[]);
        let second = assess_prospective_outcome_evidence_v0(&plans, "KRW:BTC", &[]);
        assert_eq!(first, second);
        assert_eq!(
            aggregate_prospective_opening_readiness_v0(&[
                ProspectiveOpeningReadinessV0::TimeMatureOutcomeRowsMissing,
                ProspectiveOpeningReadinessV0::AwaitingTimeMaturity,
            ]),
            ProspectiveOpeningReadinessV0::AwaitingTimeMaturity
        );
        assert!(!registration.network_execution_allowed_this_sprint);
        assert!(!registration.label_access_allowed_this_sprint);
        assert!(!registration.reward_application_allowed);
    }
}
