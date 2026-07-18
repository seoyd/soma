//! Offline, external scope attestations for immutable learned-agent opinions.

use std::{fs, path::Path};

use crate::{
    core::stable_hash_string,
    data::{
        DataSnapshot, historical_replay_dataset_digest_v0, snapshot_id_from_semantic_digest_v1,
    },
};

use super::{
    AgentOpinionRelationshipV0, BtcHistoricalRegimeConfigV0, BtcHistoricalRegimeV0,
    BtcTemporalRegimeRefV0, CYCLE_RISK_SHADOW_AGENT_ID_V0, CycleRiskOpinionAdapterContextV0,
    CycleRiskShadowConfigV0, EvidenceUsageClassV0, HistoricalEvidencePolicyV0,
    LearnedAgentObjectiveV0, MomentumCandleV0, MomentumLearningCampaignConfigV0,
    TemporalRegimeSegmentationPolicyV0, append_shadow_deliberation_v0,
    assess_momentum_campaign_sufficiency_v0, build_momentum_features_v0,
    build_momentum_learning_windows_v0, build_momentum_sequence_examples_v0,
    close_btc_temporal_regime_result_v0, freeze_btc_historical_regime_packs_v0,
    frozen_mamba3_encoder_from_seed_v0, new_shadow_deliberation_ledger_v0,
    reconstruct_cycle_risk_opinion_from_regime_v0, replay_btc_shadow_deliberations_v0,
    run_btc_historical_regime_campaigns_v0, run_cycle_risk_shadow_regime_v0,
    segment_btc_historical_regimes_v0,
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
                                            Err(_) => {
                                                record_execution_stage_v2(
                                                    &mut momentum_trace,
                                                    JointParticipantExecutionStageV2::ResultClosure,
                                                    JointExecutionStageStatusV2::Failed,
                                                    Some("joint_v2_result_closure_failed"),
                                                    vec![],
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
    };

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
}
