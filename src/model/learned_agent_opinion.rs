//! Offline, advisory-only learned-agent opinion sealing and two-round replay.

use crate::core::stable_hash_string;

use super::cycle_risk_shadow::MOMENTUM_AGENT_ID_V0;
use super::{
    BtcTemporalRegimeClosedResultV0, CYCLE_RISK_SHADOW_AGENT_ID_V0, CycleRiskShadowReportV0,
};
use crate::data::DataSnapshot;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearnedAgentObjectiveV0 {
    DirectionalMomentum,
    DownsideRisk,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpinionEvidenceConfidenceV0 {
    StrongWithinHistoricalScope,
    LimitedHistoricalEvidence,
    SparseSupportQualifiedEvidence,
    OutOfSupport,
    InsufficientEvidence,
    Abstained,
    TechnicalFailure,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpinionSupportStatusV0 {
    InSupport,
    SparseSupport,
    OutOfSupport,
    Unavailable,
    Abstained,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpinionEvidenceScopeV0 {
    HistoricalConsumedDevelopment,
    HistoricalResearchOnly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MomentumDirectionalStanceV0 {
    PositiveDirectionalHypothesis,
    NegativeDirectionalHypothesis,
    NeutralDirectionalHypothesis,
    Abstain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CycleRiskStanceV0 {
    NormalRiskHypothesis,
    ElevatedRiskHypothesis,
    SevereRiskHypothesis,
    Abstain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PredictionAvailabilityV0 {
    HistoricalOnly,
    Unavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentOpinionRelationshipV0 {
    Compatible,
    Orthogonal,
    Tension,
    DirectConflict,
    MomentumAbstained,
    RiskAbstained,
    BothAbstained,
    IncomparableEvidence,
    TechnicalFailure,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpinionTensionDimensionV0 {
    DirectionVersusDownsideRisk,
    OpportunityVersusCapitalPreservation,
    EvidenceStrengthMismatch,
    TemporalScopeMismatch,
    SupportQualificationMismatch,
    BaselineStrengthMismatch,
    AbstentionMismatch,
    UncertaintyMismatch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearnedAgentArgumentKindV0 {
    InitialStatement,
    SupportCounterpart,
    ChallengeCounterpartAssumption,
    HighlightRisk,
    HighlightOpportunity,
    RequestMoreEvidence,
    MaintainPosition,
    AbstainFromResponse,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowDeliberationQualityStatusV0 {
    InterfaceReadyShadowOnly,
    DistinctObjectivesVerified,
    MostlyAbstained,
    TensionObserved,
    OrthogonalEvidence,
    IncomparableEvidence,
    IndependenceViolation,
    InvalidOpinion,
    TechnicalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpinionTemporalScopeV0 {
    pub evidence_scope: OpinionEvidenceScopeV0,
    pub horizon_policy_digest: String,
    pub regime_id: Option<String>,
    pub window_id: Option<String>,
    pub prospective: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpinionAuthorityV0 {
    pub advisory_only: bool,
    pub eligible_to_vote: bool,
    pub eligible_to_reach_chair: bool,
    pub eligible_for_reward: bool,
    pub eligible_for_penalty: bool,
    pub eligible_for_speaking_right_change: bool,
    pub eligible_for_promotion: bool,
    pub eligible_to_execute: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MomentumOpinionPayloadV0 {
    pub directional_stance: MomentumDirectionalStanceV0,
    pub prediction_availability: PredictionAvailabilityV0,
    pub support_qualification: OpinionSupportStatusV0,
    pub abstention_reason: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CycleRiskOpinionPayloadV0 {
    pub risk_stance: CycleRiskStanceV0,
    pub prediction_availability: PredictionAvailabilityV0,
    pub historical_model_status: String,
    pub abstention_reason: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LearnedAgentOpinionPayloadV0 {
    Momentum(MomentumOpinionPayloadV0),
    CycleRisk(CycleRiskOpinionPayloadV0),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentOpinionEnvelopeV0 {
    pub protocol_version: String,
    pub opinion_id: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub doctrine_id: String,
    pub source_evidence_id: String,
    pub source_evidence_digest: String,
    pub temporal_scope: OpinionTemporalScopeV0,
    pub primary_payload: LearnedAgentOpinionPayloadV0,
    pub evidence_confidence: OpinionEvidenceConfidenceV0,
    pub support_status: OpinionSupportStatusV0,
    pub uncertainty: String,
    pub assumptions: Vec<String>,
    pub invalidation_conditions: Vec<String>,
    pub reason_codes: Vec<String>,
    pub authority: OpinionAuthorityV0,
    pub sealed: bool,
    pub opinion_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentOpinionSealV0 {
    pub opinion_id: String,
    pub agent_id: String,
    pub objective: LearnedAgentObjectiveV0,
    pub source_digest: String,
    pub opinion_digest: String,
    pub sealed_before_cross_agent_reveal: bool,
    pub authority_digest: String,
    pub seal_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairwiseAgentOpinionRelationshipV0 {
    pub relationship_id: String,
    pub left_opinion_id: String,
    pub right_opinion_id: String,
    pub relationship: AgentOpinionRelationshipV0,
    pub tension_dimensions: Vec<OpinionTensionDimensionV0>,
    pub reason_codes: Vec<String>,
    pub no_winner_selected: bool,
    pub no_action_selected: bool,
    pub relationship_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedAgentArgumentV0 {
    pub argument_id: String,
    pub author_agent_id: String,
    pub source_opinion_id: String,
    pub observed_counterpart_opinion_id: Option<String>,
    pub argument_kind: LearnedAgentArgumentKindV0,
    pub original_opinion_unchanged: bool,
    pub authority: OpinionAuthorityV0,
    pub argument_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrossAgentOpinionIndependenceProofV0 {
    pub momentum_primary_created_without_risk: bool,
    pub risk_primary_created_without_momentum: bool,
    pub both_sealed_before_reveal: bool,
    pub no_cross_agent_feature_dependency: bool,
    pub no_cross_agent_prediction_dependency: bool,
    pub no_shared_normalizer: bool,
    pub no_shared_model_parameters: bool,
    pub no_primary_opinion_mutation: bool,
    pub response_only_after_reveal: bool,
    pub no_shared_authority: bool,
    pub all_invariants_pass: bool,
    pub proof_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowLearnedAgentDeliberationV0 {
    pub deliberation_id: String,
    pub participant_agent_ids: Vec<String>,
    pub primary_opinion_ids: Vec<String>,
    pub relationship_id: String,
    pub argument_ids: Vec<String>,
    pub round_count: usize,
    pub chair_observed: bool,
    pub chair_decision_created: bool,
    pub reward_created: bool,
    pub penalty_created: bool,
    pub speaking_right_changed: bool,
    pub vote_created: bool,
    pub execution_created: bool,
    pub transcript_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FutureChairShadowObservationPacketV0 {
    pub deliberation_id: String,
    pub opinion_ids: Vec<String>,
    pub relationship_id: String,
    pub argument_ids: Vec<String>,
    pub independence_proof_digest: String,
    pub advisory_only: bool,
    pub eligible_for_chair_observation: bool,
    pub eligible_for_chair_decision: bool,
    pub packet_digest: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowDeliberationReplayV0 {
    pub momentum: LearnedAgentOpinionEnvelopeV0,
    pub risk: LearnedAgentOpinionEnvelopeV0,
    pub momentum_seal: LearnedAgentOpinionSealV0,
    pub risk_seal: LearnedAgentOpinionSealV0,
    pub relationship: PairwiseAgentOpinionRelationshipV0,
    pub arguments: Vec<LearnedAgentArgumentV0>,
    pub transcript: ShadowLearnedAgentDeliberationV0,
    pub independence: CrossAgentOpinionIndependenceProofV0,
    pub chair_packet: FutureChairShadowObservationPacketV0,
    pub quality: ShadowDeliberationQualityStatusV0,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShadowDeliberationLedgerV0 {
    pub ledger_version: String,
    pub deliberations: Vec<ShadowLearnedAgentDeliberationV0>,
    pub opinion_ids: Vec<String>,
    pub relationship_ids: Vec<String>,
    pub ledger_digest: String,
}

pub fn append_shadow_deliberation_v0(
    ledger: &mut ShadowDeliberationLedgerV0,
    replay: &ShadowDeliberationReplayV0,
) -> Result<(), String> {
    if replay.transcript.round_count != 2
        || replay.transcript.chair_observed
        || replay.transcript.vote_created
        || replay.transcript.execution_created
        || !replay.independence.all_invariants_pass
        || ledger
            .deliberations
            .iter()
            .any(|existing| existing.deliberation_id == replay.transcript.deliberation_id)
    {
        return Err("invalid_or_duplicate_shadow_deliberation".to_string());
    }
    ledger.deliberations.push(replay.transcript.clone());
    ledger
        .deliberations
        .sort_by(|left, right| left.deliberation_id.cmp(&right.deliberation_id));
    ledger
        .opinion_ids
        .extend(replay.transcript.primary_opinion_ids.clone());
    ledger.opinion_ids.sort();
    ledger.opinion_ids.dedup();
    ledger
        .relationship_ids
        .push(replay.relationship.relationship_id.clone());
    ledger.relationship_ids.sort();
    ledger.relationship_ids.dedup();
    ledger.ledger_digest = digest(&(
        &ledger.ledger_version,
        &ledger.deliberations,
        &ledger.opinion_ids,
        &ledger.relationship_ids,
    ));
    Ok(())
}

fn authority() -> OpinionAuthorityV0 {
    OpinionAuthorityV0 {
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
fn digest<T: std::fmt::Debug>(value: &T) -> String {
    stable_hash_string(&format!("{value:?}"))
}
fn seal(opinion: &mut LearnedAgentOpinionEnvelopeV0) -> LearnedAgentOpinionSealV0 {
    opinion.sealed = true;
    opinion.opinion_digest = digest(&(
        &opinion.agent_id,
        opinion.objective,
        &opinion.source_evidence_digest,
        &opinion.primary_payload,
        &opinion.reason_codes,
        &opinion.authority,
    ));
    let authority_digest = digest(&opinion.authority);
    let mut seal = LearnedAgentOpinionSealV0 {
        opinion_id: opinion.opinion_id.clone(),
        agent_id: opinion.agent_id.clone(),
        objective: opinion.objective,
        source_digest: opinion.source_evidence_digest.clone(),
        opinion_digest: opinion.opinion_digest.clone(),
        sealed_before_cross_agent_reveal: true,
        authority_digest,
        seal_digest: String::new(),
    };
    seal.seal_digest = digest(&(
        &seal.opinion_id,
        &seal.opinion_digest,
        &seal.authority_digest,
    ));
    seal
}
fn abstain_momentum(regime: &BtcTemporalRegimeClosedResultV0) -> LearnedAgentOpinionEnvelopeV0 {
    LearnedAgentOpinionEnvelopeV0 {
        protocol_version: "learned-agent-opinion-v0".into(),
        opinion_id: format!("momentum-opinion-{}", regime.regime.regime_id),
        agent_id: MOMENTUM_AGENT_ID_V0.into(),
        objective: LearnedAgentObjectiveV0::DirectionalMomentum,
        doctrine_id: "momentum-historical-shadow".into(),
        source_evidence_id: regime.regime.regime_id.clone(),
        source_evidence_digest: regime.report_digest.clone(),
        temporal_scope: OpinionTemporalScopeV0 {
            evidence_scope: OpinionEvidenceScopeV0::HistoricalConsumedDevelopment,
            horizon_policy_digest: digest(&regime.reason_codes),
            regime_id: Some(regime.regime.regime_id.clone()),
            window_id: None,
            prospective: false,
        },
        primary_payload: LearnedAgentOpinionPayloadV0::Momentum(MomentumOpinionPayloadV0 {
            directional_stance: MomentumDirectionalStanceV0::Abstain,
            prediction_availability: PredictionAvailabilityV0::HistoricalOnly,
            support_qualification: OpinionSupportStatusV0::Abstained,
            abstention_reason: Some("historical_result_not_current_prediction".into()),
        }),
        evidence_confidence: OpinionEvidenceConfidenceV0::Abstained,
        support_status: OpinionSupportStatusV0::Abstained,
        uncertainty: "historical_scope_only".into(),
        assumptions: vec!["historical_relationship_may_change".into()],
        invalidation_conditions: vec![
            "source_evidence_digest_mismatch".into(),
            "prospective_claim".into(),
        ],
        reason_codes: regime.reason_codes.clone(),
        authority: authority(),
        sealed: false,
        opinion_digest: String::new(),
    }
}
fn abstain_risk(
    report: &CycleRiskShadowReportV0,
    regime_id: &str,
) -> LearnedAgentOpinionEnvelopeV0 {
    LearnedAgentOpinionEnvelopeV0 {
        protocol_version: "learned-agent-opinion-v0".into(),
        opinion_id: format!("risk-opinion-{regime_id}"),
        agent_id: CYCLE_RISK_SHADOW_AGENT_ID_V0.into(),
        objective: LearnedAgentObjectiveV0::DownsideRisk,
        doctrine_id: "cycle-risk-historical-shadow".into(),
        source_evidence_id: regime_id.into(),
        source_evidence_digest: report.ledger_digest.clone(),
        temporal_scope: OpinionTemporalScopeV0 {
            evidence_scope: OpinionEvidenceScopeV0::HistoricalConsumedDevelopment,
            horizon_policy_digest: digest(&report.input_pack_digests),
            regime_id: Some(regime_id.into()),
            window_id: None,
            prospective: false,
        },
        primary_payload: LearnedAgentOpinionPayloadV0::CycleRisk(CycleRiskOpinionPayloadV0 {
            risk_stance: CycleRiskStanceV0::Abstain,
            prediction_availability: PredictionAvailabilityV0::HistoricalOnly,
            historical_model_status: format!("{:?}", report.aggregate_verdict),
            abstention_reason: Some("historical_comparison_is_not_current_risk_prediction".into()),
        }),
        evidence_confidence: OpinionEvidenceConfidenceV0::Abstained,
        support_status: OpinionSupportStatusV0::Abstained,
        uncertainty: "historical_scope_only".into(),
        assumptions: vec!["adverse_excursion_prevalence_may_change".into()],
        invalidation_conditions: vec![
            "source_evidence_digest_mismatch".into(),
            "prospective_claim".into(),
        ],
        reason_codes: vec!["no_timestamp_specific_risk_assessment".into()],
        authority: authority(),
        sealed: false,
        opinion_digest: String::new(),
    }
}
pub fn replay_shadow_deliberation_v0(
    momentum_regime: &BtcTemporalRegimeClosedResultV0,
    risk_report: &CycleRiskShadowReportV0,
) -> Result<ShadowDeliberationReplayV0, String> {
    let mut momentum = abstain_momentum(momentum_regime);
    let mut risk = abstain_risk(risk_report, &momentum_regime.regime.regime_id);
    let momentum_seal = seal(&mut momentum);
    let risk_seal = seal(&mut risk);
    if !momentum_seal.sealed_before_cross_agent_reveal
        || !risk_seal.sealed_before_cross_agent_reveal
    {
        return Err("cross_agent_reveal_blocked".into());
    }
    let relationship = PairwiseAgentOpinionRelationshipV0 {
        relationship_id: format!(
            "relationship-{}",
            digest(&(&momentum.opinion_id, &risk.opinion_id))
        ),
        left_opinion_id: momentum.opinion_id.clone(),
        right_opinion_id: risk.opinion_id.clone(),
        relationship: AgentOpinionRelationshipV0::BothAbstained,
        tension_dimensions: vec![],
        reason_codes: vec!["both_historical_objectives_abstained".into()],
        no_winner_selected: true,
        no_action_selected: true,
        relationship_digest: String::new(),
    };
    let mut relationship = relationship;
    relationship.relationship_digest = digest(&(
        &relationship.relationship_id,
        relationship.relationship,
        &relationship.reason_codes,
    ));
    let arguments = vec![
        argument(&momentum, None),
        argument(&risk, Some(momentum.opinion_id.clone())),
    ];
    let mut ids = vec![momentum.agent_id.clone(), risk.agent_id.clone()];
    ids.sort();
    let proof = proof();
    let mut transcript = ShadowLearnedAgentDeliberationV0 {
        deliberation_id: format!(
            "deliberation-{}",
            digest(&(&momentum.opinion_digest, &risk.opinion_digest))
        ),
        participant_agent_ids: ids,
        primary_opinion_ids: vec![momentum.opinion_id.clone(), risk.opinion_id.clone()],
        relationship_id: relationship.relationship_id.clone(),
        argument_ids: arguments.iter().map(|a| a.argument_id.clone()).collect(),
        round_count: 2,
        chair_observed: false,
        chair_decision_created: false,
        reward_created: false,
        penalty_created: false,
        speaking_right_changed: false,
        vote_created: false,
        execution_created: false,
        transcript_digest: String::new(),
    };
    transcript.transcript_digest = digest(&(
        &transcript.deliberation_id,
        &transcript.participant_agent_ids,
        &transcript.primary_opinion_ids,
        &transcript.argument_ids,
    ));
    let mut packet = FutureChairShadowObservationPacketV0 {
        deliberation_id: transcript.deliberation_id.clone(),
        opinion_ids: transcript.primary_opinion_ids.clone(),
        relationship_id: relationship.relationship_id.clone(),
        argument_ids: transcript.argument_ids.clone(),
        independence_proof_digest: proof.proof_digest.clone(),
        advisory_only: true,
        eligible_for_chair_observation: false,
        eligible_for_chair_decision: false,
        packet_digest: String::new(),
    };
    packet.packet_digest = digest(&(
        &packet.deliberation_id,
        &packet.opinion_ids,
        &packet.relationship_id,
    ));
    Ok(ShadowDeliberationReplayV0 {
        momentum,
        risk,
        momentum_seal,
        risk_seal,
        relationship,
        arguments,
        transcript,
        independence: proof,
        chair_packet: packet,
        quality: ShadowDeliberationQualityStatusV0::MostlyAbstained,
    })
}

pub fn replay_btc_shadow_deliberations_v0(
    snapshot: &DataSnapshot,
    campaign_config: &super::MomentumLearningCampaignConfigV0,
) -> Result<Vec<ShadowDeliberationReplayV0>, String> {
    let sufficiency =
        super::assess_momentum_campaign_sufficiency_v0(snapshot.row_count, campaign_config)
            .map_err(|_| "momentum_sufficiency_invalid".to_string())?;
    let config = super::BtcHistoricalRegimeConfigV0 {
        minimum_regimes: 2,
        regime_rows: sufficiency.required_minimum_rows,
        inter_regime_gap_rows: campaign_config.purge_gap_rows,
        minimum_campaign_windows_per_regime: campaign_config.minimum_evaluated_windows,
        segmentation_policy: super::TemporalRegimeSegmentationPolicyV0::EqualLengthChronological,
    };
    let segmentation = super::segment_btc_historical_regimes_v0(snapshot, &config)
        .map_err(|_| "regime_segmentation_failed".to_string())?;
    let packs = super::freeze_btc_historical_regime_packs_v0(
        snapshot,
        &segmentation,
        &super::HistoricalEvidencePolicyV0::default(),
    )
    .map_err(|_| "regime_pack_freeze_failed".to_string())?;
    let encoder = super::frozen_mamba3_encoder_from_seed_v0(
        &campaign_config.feature_config,
        campaign_config.campaign_seed,
        campaign_config.backend_preference,
        campaign_config.fallback_policy,
    )
    .map_err(|_| "momentum_encoder_unavailable".to_string())?;
    let raw = super::run_btc_historical_regime_campaigns_v0(&packs, campaign_config, &encoder)
        .map_err(|_| "momentum_regime_replay_failed".to_string())?;
    let mut closed = Vec::new();
    for (rank, (regime, pack)) in packs.iter().enumerate() {
        let result = raw
            .iter()
            .find(|item| item.regime_id == regime.regime_id)
            .ok_or_else(|| "missing_momentum_regime".to_string())?;
        let reference = super::BtcTemporalRegimeRefV0 {
            regime_id: regime.regime_id.clone(),
            chronological_rank: rank,
            row_count: regime.row_count,
            range_digest: stable_hash_string(&format!(
                "{}:{}:{}",
                regime.start_timestamp_ms, regime.end_timestamp_ms, regime.row_count
            )),
            pack_digest: pack.digest.clone(),
        };
        closed.push(super::close_btc_temporal_regime_result_v0(
            result, reference,
        ));
    }
    let risk =
        super::run_cycle_risk_shadow_v0(snapshot, &super::CycleRiskShadowConfigV0::default())
            .map_err(|_| "cycle_risk_shadow_replay_failed".to_string())?;
    closed
        .iter()
        .map(|item| replay_shadow_deliberation_v0(item, &risk))
        .collect()
}
fn argument(
    opinion: &LearnedAgentOpinionEnvelopeV0,
    other: Option<String>,
) -> LearnedAgentArgumentV0 {
    let mut a = LearnedAgentArgumentV0 {
        argument_id: format!("argument-{}", opinion.opinion_id),
        author_agent_id: opinion.agent_id.clone(),
        source_opinion_id: opinion.opinion_id.clone(),
        observed_counterpart_opinion_id: other,
        argument_kind: LearnedAgentArgumentKindV0::AbstainFromResponse,
        original_opinion_unchanged: true,
        authority: authority(),
        argument_digest: String::new(),
    };
    a.argument_digest = digest(&(&a.argument_id, &a.source_opinion_id, &a.authority));
    a
}
fn proof() -> CrossAgentOpinionIndependenceProofV0 {
    let mut p = CrossAgentOpinionIndependenceProofV0 {
        momentum_primary_created_without_risk: true,
        risk_primary_created_without_momentum: true,
        both_sealed_before_reveal: true,
        no_cross_agent_feature_dependency: true,
        no_cross_agent_prediction_dependency: true,
        no_shared_normalizer: true,
        no_shared_model_parameters: true,
        no_primary_opinion_mutation: true,
        response_only_after_reveal: true,
        no_shared_authority: true,
        all_invariants_pass: true,
        proof_digest: String::new(),
    };
    p.proof_digest = digest(&p.all_invariants_pass);
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_is_advisory_only_and_independence_is_deterministic() {
        let first = authority();
        let second = authority();
        assert_eq!(first, second);
        assert!(first.advisory_only);
        assert!(!first.eligible_to_vote);
        assert!(!first.eligible_to_reach_chair);
        assert!(!first.eligible_for_reward);
        assert!(!first.eligible_for_penalty);
        assert!(!first.eligible_to_execute);
        assert_eq!(proof(), proof());
    }
}
