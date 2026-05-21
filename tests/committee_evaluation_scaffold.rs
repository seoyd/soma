use soma_zero::{
    CommitteeDecision, CommitteeDecisionRecord, CommitteeFinalAction, CommitteeOutcome,
    EvidenceSourceKind, PersonaHorizon, PersonaStance, PersonaVote, ReasonCode, Regime,
    RiskDecision, RiskDecisionKind, build_committee_evaluation_scaffold,
};

fn outcome(
    persona_id: &str,
    stance: PersonaStance,
    doctrine_violations: Vec<&str>,
    final_action: CommitteeFinalAction,
) -> CommitteeOutcome {
    CommitteeOutcome {
        committee_record: CommitteeDecisionRecord {
            decision_id: format!("decision-{persona_id}"),
            symbol: "BTC-KRW".to_string(),
            timestamp_ms: 1_700_000_000_000,
            selected_speakers: vec![persona_id.to_string()],
            all_votes: vec![PersonaVote {
                persona_id: persona_id.to_string(),
                stance,
                conviction: 0.8,
                voice_power: 0.7,
                horizon: PersonaHorizon::Swing,
                source_kind: EvidenceSourceKind::OfficialApiCollected,
                regime_fit: 0.9,
                data_quality_fit: 0.9,
                risk_fit: 0.8,
                expected_edge_fit: 0.7,
                doctrine_violations: doctrine_violations
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                reason_codes: vec![ReasonCode::PersonaVoteBuilt],
            }],
            weighted_score: 0.4,
            disagreement_score: 0.2,
            groupthink_risk: 0.3,
            uncertainty: 0.3,
            final_decision: CommitteeDecision::ApproveCandidate,
            chair_reason_codes: vec![ReasonCode::ChairV0Built],
            source_kind: EvidenceSourceKind::OfficialApiCollected,
            regime: Regime::TrendUp,
            core_fingerprint: None,
            reason_codes: vec![ReasonCode::ChairV0Built],
        },
        risk_decision: RiskDecision {
            kind: RiskDecisionKind::ApprovePaper,
            approved_order_plan: None,
            reason_codes: vec![ReasonCode::ApprovePaperOnly],
            audit_id: "audit".to_string(),
        },
        final_action,
        reason_codes: vec![ReasonCode::CommitteeRiskBridgeBuilt],
    }
}

#[test]
fn evaluation_scaffold_counts_stances_and_tracks_no_trade_value() {
    let scaffold = build_committee_evaluation_scaffold(&[
        outcome(
            "defensive_value_risk",
            PersonaStance::NoTrade,
            vec![],
            CommitteeFinalAction::FinalNoTrade,
        ),
        outcome(
            "defensive_value_risk",
            PersonaStance::Veto,
            vec!["quality-hard-stop"],
            CommitteeFinalAction::FinalDenied,
        ),
        outcome(
            "trend_breakout_fast",
            PersonaStance::Approve,
            vec![],
            CommitteeFinalAction::PaperApprove,
        ),
    ]);
    let defensive = scaffold
        .persona_metrics
        .iter()
        .find(|metric| metric.persona_id == "defensive_value_risk")
        .expect("defensive metric");
    assert_eq!(defensive.sample_count, 2);
    assert_eq!(defensive.stance_counts.get("NoTrade").copied(), Some(1));
    assert_eq!(defensive.stance_counts.get("Veto").copied(), Some(1));
    assert_eq!(defensive.doctrine_violation_count, 1);
    assert!(defensive.no_trade_value_proxy.expect("proxy") > 0.0);
}

#[test]
fn evaluation_scaffold_is_conservative_on_small_samples() {
    let scaffold = build_committee_evaluation_scaffold(&[outcome(
        "trend_breakout_fast",
        PersonaStance::Approve,
        vec![],
        CommitteeFinalAction::PaperApprove,
    )]);
    assert!(!scaffold.enough_samples);
    assert_eq!(
        scaffold.recommendation,
        soma_zero::CommitteeEvaluationRecommendation::NotEnoughSamples
    );
}
