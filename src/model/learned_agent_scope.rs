//! Offline, external scope attestations for immutable learned-agent opinions.

use crate::{core::stable_hash_string, data::DataSnapshot};

use super::{
    AgentOpinionRelationshipV0, BtcHistoricalRegimeConfigV0, CycleRiskShadowConfigV0,
    LearnedAgentObjectiveV0, MomentumLearningCampaignConfigV0, TemporalRegimeSegmentationPolicyV0,
    append_shadow_deliberation_v0, assess_momentum_campaign_sufficiency_v0,
    new_shadow_deliberation_ledger_v0, replay_btc_shadow_deliberations_v0,
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
        .take(momentum_raw.len())
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
    for replay in replays.iter().take(momentum_raw.len()) {
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
            .take(momentum_raw.len())
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
}
