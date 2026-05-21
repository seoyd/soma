use soma_zero::{
    ChairDiagnosticStatus, ChairV0, CommitteeInput, EvidenceSourceKind, PersonaHorizon,
    PersonaScoringInput, PersonaStance, PersonaVote, ProviderMarket, ReasonCode, Regime,
    SignalOutput, build_chair_diagnostics,
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
            p_win: 0.60,
            p_stop: 0.30,
            expected_return: 0.01,
            expected_drawdown: 0.02,
            confidence: 0.75,
            no_trade_probability: 0.25,
            source: "test".to_string(),
        },
        data_quality_score: 0.90,
        spread_bps: Some(6.0),
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
        expected_edge_fit: 0.8,
        doctrine_violations: doctrine_violations
            .into_iter()
            .map(str::to_string)
            .collect(),
        reason_codes: vec![ReasonCode::PersonaVoteBuilt],
    }
}

#[test]
fn selected_trace_includes_final_voice_power() {
    let input = CommitteeInput {
        scoring_input: scoring_input(
            PersonaHorizon::Swing,
            EvidenceSourceKind::OfficialApiCollected,
        ),
        persona_votes: vec![vote(
            "trend_breakout_fast",
            PersonaStance::Approve,
            PersonaHorizon::Intraday,
            EvidenceSourceKind::OfficialApiCollected,
            vec![],
        )],
        target_horizon: PersonaHorizon::Swing,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    };
    let record = ChairV0::default().evaluate(&input);
    let diagnostics = build_chair_diagnostics(&input, &record);
    assert!(diagnostics.speaker_traces[0].final_voice_power > 0.0);
}

#[test]
fn filter_reasons_are_recorded() {
    let input = CommitteeInput {
        scoring_input: scoring_input(
            PersonaHorizon::Intraday,
            EvidenceSourceKind::OfficialApiCollected,
        ),
        persona_votes: vec![
            vote(
                "quality_value_long",
                PersonaStance::Approve,
                PersonaHorizon::LongTerm,
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
    };
    let record = ChairV0::default().evaluate(&input);
    let diagnostics = build_chair_diagnostics(&input, &record);
    let reasons = diagnostics
        .speaker_traces
        .iter()
        .flat_map(|trace| trace.filter_reasons.iter())
        .collect::<Vec<_>>();
    assert!(reasons.contains(&&soma_zero::SpeakerFilterReason::Inactive));
    assert!(reasons.contains(&&soma_zero::SpeakerFilterReason::SourceIncompatible));
    assert!(reasons.contains(&&soma_zero::SpeakerFilterReason::HorizonIncompatible));
}

#[test]
fn cluster_penalty_and_contrarian_show_up_without_changing_decision() {
    let input = CommitteeInput {
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
                "cycle_regime_guard",
                PersonaStance::Approve,
                PersonaHorizon::MultiDay,
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
        ],
        target_horizon: PersonaHorizon::Swing,
        source_kind: EvidenceSourceKind::OfficialApiCollected,
        regime: Regime::TrendUp,
        reason_codes: vec![],
    };
    let record = ChairV0::default().evaluate(&input);
    let diagnostics = build_chair_diagnostics(&input, &record);
    assert_eq!(diagnostics.final_decision, record.final_decision);
    assert!(diagnostics.cluster_penalty_applied);
    assert!(
        diagnostics.contrarian_included
            || diagnostics.diagnostic_status == ChairDiagnosticStatus::GroupthinkRisk
    );
}
