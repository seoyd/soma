use soma_zero::{
    CommitteeFinalAction, CommitteeReplayRecord, CommitteeReplayReport, PersonaConflictStatus,
    PersonaHorizon, PersonaStance, PersonaVote, ReasonCode, build_persona_conflict_matrix,
};

fn replay_report(votes: Vec<Vec<PersonaVote>>) -> CommitteeReplayReport {
    let record_count = votes.len();
    CommitteeReplayReport {
        replay_id: "conflict".to_string(),
        records: votes
            .into_iter()
            .enumerate()
            .map(|(index, persona_votes)| CommitteeReplayRecord {
                scenario_row: soma_zero::CommitteeScenarioRow {
                    scenario_row_id: format!("row-{index}"),
                    symbol: "BTC-KRW".to_string(),
                    timestamp_ms: 1_700_000_000_000 + index as u64,
                    source_kind: soma_zero::CommitteeScenarioSourceKind::Fixture,
                    evidence_source_kind: soma_zero::EvidenceSourceKind::TestFixture,
                    market: soma_zero::ProviderMarket::Crypto,
                    target_horizon: PersonaHorizon::Swing,
                    feature_vector: None,
                    regime: soma_zero::Regime::TrendUp,
                    signal_summary: "test".to_string(),
                    data_quality_score: 0.9,
                    spread_bps: Some(6.0),
                    expected_edge_after_cost: 0.01,
                    expected_drawdown: 0.02,
                    risk_snapshot_summary: None,
                    provenance_summary: "fixture".to_string(),
                    benchmark_status: None,
                    baseline_signal_summary: None,
                    external_prediction_summary: None,
                    no_trade_counterfactual: None,
                    risk_denial_counterfactual: None,
                    outcome_reference: None,
                    materialization_level:
                        soma_zero::CommitteeScenarioMaterializationLevel::Fixture,
                    materialization_confidence: 0.9,
                    reason_codes: vec![ReasonCode::SummaryDerived],
                },
                persona_votes: persona_votes.clone(),
                chair_decision_record: soma_zero::CommitteeDecisionRecord {
                    decision_id: format!("decision-{index}"),
                    symbol: "BTC-KRW".to_string(),
                    timestamp_ms: 1_700_000_000_000 + index as u64,
                    selected_speakers: vec![],
                    all_votes: persona_votes,
                    weighted_score: 0.0,
                    disagreement_score: 0.0,
                    groupthink_risk: 0.0,
                    uncertainty: 0.0,
                    final_decision: soma_zero::CommitteeDecision::NoTrade,
                    chair_reason_codes: vec![ReasonCode::ChairV0Built],
                    source_kind: soma_zero::EvidenceSourceKind::TestFixture,
                    regime: soma_zero::Regime::TrendUp,
                    core_fingerprint: None,
                    reason_codes: vec![ReasonCode::ChairV0Built],
                },
                risk_bridge_outcome: soma_zero::CommitteeOutcome {
                    committee_record: soma_zero::CommitteeDecisionRecord {
                        decision_id: format!("decision-{index}"),
                        symbol: "BTC-KRW".to_string(),
                        timestamp_ms: 1_700_000_000_000 + index as u64,
                        selected_speakers: vec![],
                        all_votes: vec![],
                        weighted_score: 0.0,
                        disagreement_score: 0.0,
                        groupthink_risk: 0.0,
                        uncertainty: 0.0,
                        final_decision: soma_zero::CommitteeDecision::NoTrade,
                        chair_reason_codes: vec![ReasonCode::ChairV0Built],
                        source_kind: soma_zero::EvidenceSourceKind::TestFixture,
                        regime: soma_zero::Regime::TrendUp,
                        core_fingerprint: None,
                        reason_codes: vec![ReasonCode::ChairV0Built],
                    },
                    risk_decision: soma_zero::RiskDecision {
                        kind: soma_zero::RiskDecisionKind::Deny,
                        approved_order_plan: None,
                        reason_codes: vec![ReasonCode::NoTradePreferred],
                        audit_id: "audit".to_string(),
                    },
                    final_action: CommitteeFinalAction::FinalNoTrade,
                    reason_codes: vec![ReasonCode::CommitteeRiskBridgeBuilt],
                },
                final_action: CommitteeFinalAction::FinalNoTrade,
                replay_fingerprint: format!("fp-{index}"),
                reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
            })
            .collect(),
        record_count,
        source_summary: "Fixture".to_string(),
        final_action_counts: std::collections::BTreeMap::new(),
        risk_denial_counts: std::collections::BTreeMap::new(),
        chair_decision_counts: std::collections::BTreeMap::new(),
        deterministic_fingerprint: "fp".to_string(),
        reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
    }
}

fn vote(persona_id: &str, stance: PersonaStance) -> PersonaVote {
    PersonaVote {
        persona_id: persona_id.to_string(),
        stance,
        conviction: 0.8,
        voice_power: 0.7,
        horizon: PersonaHorizon::Swing,
        source_kind: soma_zero::EvidenceSourceKind::TestFixture,
        regime_fit: 0.9,
        data_quality_fit: 0.9,
        risk_fit: 0.8,
        expected_edge_fit: 0.8,
        doctrine_violations: vec![],
        reason_codes: vec![ReasonCode::PersonaVoteBuilt],
    }
}

#[test]
fn matrix_counts_alignment_and_opposition() {
    let report = replay_report(vec![
        vec![
            vote("a", PersonaStance::Approve),
            vote("b", PersonaStance::Approve),
        ],
        vec![
            vote("a", PersonaStance::Approve),
            vote("b", PersonaStance::NoTrade),
        ],
        vec![
            vote("a", PersonaStance::NoTrade),
            vote("b", PersonaStance::Approve),
        ],
    ]);
    let matrix = build_persona_conflict_matrix(&report);
    assert_eq!(matrix.pairs[0].same_stance_count, 1);
    assert_eq!(matrix.pairs[0].opposite_stance_count, 2);
}

#[test]
fn matrix_statuses_cover_alignment_conflict_and_small_samples() {
    let aligned = build_persona_conflict_matrix(&replay_report(vec![
        vec![
            vote("a", PersonaStance::Approve),
            vote("b", PersonaStance::Approve),
        ],
        vec![
            vote("a", PersonaStance::Approve),
            vote("b", PersonaStance::Approve),
        ],
        vec![
            vote("a", PersonaStance::Approve),
            vote("b", PersonaStance::Approve),
        ],
    ]));
    let conflicted = build_persona_conflict_matrix(&replay_report(vec![
        vec![
            vote("a", PersonaStance::Approve),
            vote("b", PersonaStance::Veto),
        ],
        vec![
            vote("a", PersonaStance::Veto),
            vote("b", PersonaStance::Approve),
        ],
        vec![
            vote("a", PersonaStance::Approve),
            vote("b", PersonaStance::Veto),
        ],
    ]));
    let small = build_persona_conflict_matrix(&replay_report(vec![vec![
        vote("a", PersonaStance::Approve),
        vote("b", PersonaStance::Approve),
    ]]));
    assert_eq!(aligned.conflict_status, PersonaConflictStatus::TooAligned);
    assert_eq!(
        conflicted.conflict_status,
        PersonaConflictStatus::TooConflicted
    );
    assert_eq!(
        small.conflict_status,
        PersonaConflictStatus::InsufficientSamples
    );
}
