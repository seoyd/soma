use soma_zero::{
    ChairV0, CommitteeDecision, CommitteeInput, EvidenceSourceKind, PersonaHorizon,
    PersonaScoringInput, PersonaStance, PersonaVote, ProviderMarket, ReasonCode, Regime,
    SignalOutput,
};

fn scoring_input(
    target_horizon: PersonaHorizon,
    source_kind: EvidenceSourceKind,
) -> PersonaScoringInput {
    PersonaScoringInput {
        symbol: "BTC-KRW".to_string(),
        timestamp_ms: 1_700_000_000_000,
        source_kind,
        market: ProviderMarket::Crypto,
        target_horizon,
        feature_vector: None,
        regime: Regime::TrendUp,
        signal_output: SignalOutput {
            symbol: "BTC-KRW".to_string(),
            horizon_bars: 12,
            p_win: 0.62,
            p_stop: 0.28,
            expected_return: 0.01,
            expected_drawdown: 0.02,
            confidence: 0.75,
            no_trade_probability: 0.25,
            source: "test".to_string(),
        },
        data_quality_score: 0.92,
        spread_bps: Some(4.0),
        expected_edge_after_cost: 0.01,
        expected_drawdown: 0.02,
        risk_snapshot: None,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn vote(
    persona_id: &str,
    stance: PersonaStance,
    horizon: PersonaHorizon,
    source_kind: EvidenceSourceKind,
    doctrine_violations: Vec<&str>,
) -> PersonaVote {
    PersonaVote {
        persona_id: persona_id.to_string(),
        stance,
        conviction: 0.8,
        voice_power: 0.7,
        horizon,
        source_kind,
        regime_fit: 0.9,
        data_quality_fit: 0.9,
        risk_fit: 0.8,
        expected_edge_fit: 0.7,
        doctrine_violations: doctrine_violations
            .into_iter()
            .map(str::to_string)
            .collect(),
        reason_codes: vec![ReasonCode::PersonaVoteBuilt],
    }
}

#[test]
fn chair_selects_active_compatible_speakers() {
    let record = ChairV0::default().evaluate(&CommitteeInput {
        scoring_input: scoring_input(
            PersonaHorizon::Swing,
            EvidenceSourceKind::OfficialApiCollected,
        ),
        persona_votes: vec![
            vote(
                "trend_breakout_fast",
                PersonaStance::Approve,
                PersonaHorizon::Intraday,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "defensive_value_risk",
                PersonaStance::ReduceSize,
                PersonaHorizon::Swing,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "quality_value_long",
                PersonaStance::Approve,
                PersonaHorizon::LongTerm,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
        ],
        target_horizon: PersonaHorizon::Swing,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    });
    assert!(
        record
            .selected_speakers
            .contains(&"trend_breakout_fast".to_string())
    );
    assert!(
        record
            .selected_speakers
            .contains(&"defensive_value_risk".to_string())
    );
    assert!(
        !record
            .selected_speakers
            .contains(&"quality_value_long".to_string())
    );
}

#[test]
fn chair_filters_incompatible_horizon_and_source() {
    let record = ChairV0::default().evaluate(&CommitteeInput {
        scoring_input: scoring_input(
            PersonaHorizon::Intraday,
            EvidenceSourceKind::OfficialApiCollected,
        ),
        persona_votes: vec![
            vote(
                "trend_breakout_fast",
                PersonaStance::Approve,
                PersonaHorizon::Intraday,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "defensive_value_risk",
                PersonaStance::Approve,
                PersonaHorizon::Swing,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "cycle_regime_guard",
                PersonaStance::Approve,
                PersonaHorizon::MultiDay,
                EvidenceSourceKind::YFinanceResearch,
                vec![],
            ),
        ],
        target_horizon: PersonaHorizon::Intraday,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    });
    assert_eq!(
        record.selected_speakers,
        vec!["trend_breakout_fast".to_string()]
    );
}

#[test]
fn hard_veto_produces_vetoed() {
    let record = ChairV0::default().evaluate(&CommitteeInput {
        scoring_input: scoring_input(
            PersonaHorizon::Swing,
            EvidenceSourceKind::OfficialApiCollected,
        ),
        persona_votes: vec![
            vote(
                "trend_breakout_fast",
                PersonaStance::Approve,
                PersonaHorizon::Intraday,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "defensive_value_risk",
                PersonaStance::Veto,
                PersonaHorizon::Swing,
                EvidenceSourceKind::OfficialApiCollected,
                vec!["quality-hard-stop"],
            ),
        ],
        target_horizon: PersonaHorizon::Swing,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    });
    assert_eq!(record.final_decision, CommitteeDecision::Vetoed);
}

#[test]
fn high_disagreement_requires_conservative_output() {
    let record = ChairV0::default().evaluate(&CommitteeInput {
        scoring_input: scoring_input(
            PersonaHorizon::Swing,
            EvidenceSourceKind::OfficialApiCollected,
        ),
        persona_votes: vec![
            vote(
                "trend_breakout_fast",
                PersonaStance::StrongApprove,
                PersonaHorizon::Intraday,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "defensive_value_risk",
                PersonaStance::NoTrade,
                PersonaHorizon::Swing,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "cycle_regime_guard",
                PersonaStance::ReduceSize,
                PersonaHorizon::MultiDay,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
        ],
        target_horizon: PersonaHorizon::Swing,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    });
    assert!(matches!(
        record.final_decision,
        CommitteeDecision::ReduceSizeCandidate | CommitteeDecision::RequireHumanConfirm
    ));
}

#[test]
fn aligned_duplicate_votes_produce_groupthink_warning_and_cluster_penalty() {
    let chair = ChairV0::default();
    let duplicate = chair.evaluate(&CommitteeInput {
        scoring_input: scoring_input(
            PersonaHorizon::Swing,
            EvidenceSourceKind::OfficialApiCollected,
        ),
        persona_votes: vec![
            vote(
                "trend_breakout_fast",
                PersonaStance::Approve,
                PersonaHorizon::Intraday,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "trend_breakout_fast",
                PersonaStance::Approve,
                PersonaHorizon::Intraday,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "trend_breakout_fast",
                PersonaStance::Approve,
                PersonaHorizon::Intraday,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
        ],
        target_horizon: PersonaHorizon::Swing,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    });
    let diverse = chair.evaluate(&CommitteeInput {
        scoring_input: scoring_input(
            PersonaHorizon::Swing,
            EvidenceSourceKind::OfficialApiCollected,
        ),
        persona_votes: vec![
            vote(
                "trend_breakout_fast",
                PersonaStance::Approve,
                PersonaHorizon::Intraday,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "defensive_value_risk",
                PersonaStance::Approve,
                PersonaHorizon::Swing,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
            vote(
                "cycle_regime_guard",
                PersonaStance::Approve,
                PersonaHorizon::MultiDay,
                EvidenceSourceKind::OfficialApiCollected,
                vec![],
            ),
        ],
        target_horizon: PersonaHorizon::Swing,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    });
    assert!(
        duplicate
            .chair_reason_codes
            .contains(&ReasonCode::GroupthinkRiskElevated)
    );
    assert!(
        duplicate
            .chair_reason_codes
            .contains(&ReasonCode::ClusterPenaltyApplied)
    );
    assert!(duplicate.groupthink_risk > diverse.groupthink_risk);
}

#[test]
fn no_valid_speakers_becomes_no_trade_and_is_deterministic() {
    let input = CommitteeInput {
        scoring_input: scoring_input(
            PersonaHorizon::LongTerm,
            EvidenceSourceKind::OfficialApiCollected,
        ),
        persona_votes: vec![vote(
            "trend_breakout_fast",
            PersonaStance::Approve,
            PersonaHorizon::Intraday,
            EvidenceSourceKind::OfficialApiCollected,
            vec![],
        )],
        target_horizon: PersonaHorizon::LongTerm,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    };
    let first = ChairV0::default().evaluate(&input);
    let second = ChairV0::default().evaluate(&input);
    assert_eq!(first.final_decision, CommitteeDecision::NoTrade);
    assert_eq!(first, second);
}
