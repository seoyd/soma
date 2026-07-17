//! Offline, external scope attestations for immutable learned-agent opinions.

use crate::{core::stable_hash_string, data::DataSnapshot};

use super::{
    AgentOpinionRelationshipV0, BtcHistoricalRegimeConfigV0, BtcTemporalRegimeRefV0,
    CYCLE_RISK_SHADOW_AGENT_ID_V0, CycleRiskOpinionAdapterContextV0, CycleRiskShadowConfigV0,
    HistoricalEvidencePolicyV0, LearnedAgentObjectiveV0, MomentumCandleV0,
    MomentumLearningCampaignConfigV0, TemporalRegimeSegmentationPolicyV0,
    append_shadow_deliberation_v0, assess_momentum_campaign_sufficiency_v0,
    build_momentum_features_v0, build_momentum_learning_windows_v0,
    build_momentum_sequence_examples_v0, close_btc_temporal_regime_result_v0,
    freeze_btc_historical_regime_packs_v0, frozen_mamba3_encoder_from_seed_v0,
    new_shadow_deliberation_ledger_v0, reconstruct_cycle_risk_opinion_from_regime_v0,
    replay_btc_shadow_deliberations_v0, run_btc_historical_regime_campaigns_v0,
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
    pub effective_anchor_scope_digest_v1: String,
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
    value.opinion_digest_v1 = stable_hash_string(&hex(&bytes));
    Ok(value)
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
            effective_anchor_scope_digest_v1: anchors.scope_digest_v1,
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
            effective_anchor_scope_digest_v1: anchors.scope_digest_v1,
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
        let candidates = risk
            .iter()
            .filter(|(risk_opinion, risk_seal)| {
                risk_opinion.sealed
                    && risk_seal.sealed_before_cross_agent_reveal
                    && risk_opinion.source_result.canonical_raw_scope_digest_v1
                        == momentum_opinion.source_result.canonical_raw_scope_digest_v1
            })
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            unmatched_momentum.push(momentum_opinion.opinion_id.clone());
            continue;
        }
        let (risk_opinion, _) = candidates[0];
        used_risk.push(risk_opinion.opinion_id.clone());
        let anchors_equal = momentum_opinion
            .source_result
            .effective_anchor_scope_digest_v1
            == risk_opinion.source_result.effective_anchor_scope_digest_v1;
        let raw = SourceBoundRawScopeAlignmentV1::ExactSameCanonicalRows;
        let anchor = if anchors_equal {
            SourceBoundAnchorAlignmentV1::ExactSameAnchors
        } else {
            SourceBoundAnchorAlignmentV1::SameRawScopeDifferentAnchors
        };
        let comparability = if anchors_equal
            && momentum_opinion.source_result.forecast_scope_digest_v1
                == risk_opinion.source_result.forecast_scope_digest_v1
        {
            SourceBoundScopeComparabilityV1::ExactDecisionScopeComparable
        } else {
            SourceBoundScopeComparabilityV1::RegimeSummaryComparableWithCaveats
        };
        let mut bytes = Vec::new();
        strv(&mut bytes, &momentum_opinion.opinion_id);
        strv(&mut bytes, &risk_opinion.opinion_id);
        tag(&mut bytes, if anchors_equal { 1 } else { 2 });
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
    let relationship_digest = stable_hash_string(&hex(&relationship_bytes));
    let mut value = SourceBoundShadowDeliberationV1 {
        deliberation_version: registration.deliberation_protocol_version.clone(),
        creation_mode: LearnedAgentOpinionCreationModeV1::HistoricalRetrospectiveSourceBoundReplay,
        protocol_registration_digest_v1: registration.policy_digest_v1,
        momentum_opinion_id: momentum.0.opinion_id.clone(),
        risk_opinion_id: risk.0.opinion_id.clone(),
        momentum_seal_digest_v1: momentum.1.seal_digest_v1.clone(),
        risk_seal_digest_v1: risk.1.seal_digest_v1.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
