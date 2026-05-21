use soma_zero::{
    CandidateEvidenceClass, CandidateLifecycleStatus, CandidateSourceKind, CommitteeCycleInput,
    CommitteeCycleOwnerContext, CommitteeCycleRunner, GeneratedCandidate,
    HumanConfirmProtocolConfig, OwnerInput, OwnerInputKind, OwnerInputStatus, OwnerInputTargetType,
    ProviderMarket, Regime, RiskSnapshot,
};

fn candidate(
    symbol: &str,
    edge: f64,
    drawdown: f64,
    confidence: f64,
    spread_bps: f64,
    trade_value: f64,
    regime: Regime,
) -> GeneratedCandidate {
    GeneratedCandidate {
        candidate_id: format!("candidate-{symbol}"),
        symbol: symbol.to_string(),
        market: ProviderMarket::USEquity,
        timeframe: "1d".to_string(),
        horizon_bars: 20,
        source_kind: CandidateSourceKind::KISOfficialEvidence,
        evidence_class: CandidateEvidenceClass::Official,
        initial_status: CandidateLifecycleStatus::EvidenceReady,
        expected_edge: Some(edge),
        expected_drawdown: Some(drawdown),
        data_quality_score: Some(0.95),
        signal_summary: Some(symbol.to_string()),
        timestamp_ms: 1715000000000,
        confidence: Some(confidence),
        spread_bps: Some(spread_bps),
        trade_value: Some(trade_value),
        regime: Some(regime),
        paper_outcome_hint: None,
        source_report_path: None,
        reason_codes: vec![],
    }
}

fn risk_snapshot() -> RiskSnapshot {
    RiskSnapshot {
        daily_pnl_pct: 0.0,
        consecutive_losses: 0,
        current_positions_count: 0,
        total_exposure_pct: 0.0,
        symbol_exposure_pct: 0.0,
        api_health_score: 1.0,
        data_quality_score: 0.95,
    }
}

fn owner_input(kind: OwnerInputKind, symbol: &str) -> OwnerInput {
    OwnerInput {
        owner_input_id: format!("{:?}-{symbol}", kind),
        timestamp_ms: Some(1715000000100),
        owner_id: Some("owner".to_string()),
        input_kind: kind,
        target_type: OwnerInputTargetType::Symbol,
        target_id: None,
        symbol: Some(symbol.to_string()),
        market: None,
        freeform_note: None,
        structured_payload: None,
        requested_action: None,
        status: OwnerInputStatus::Submitted,
        reason_codes: vec![],
    }
}

#[test]
fn committee_cycle_handles_approved_blocked_no_trade_and_owner_outcomes() {
    let approved = CommitteeCycleRunner::default()
        .run_cycle(&CommitteeCycleInput {
            candidate: candidate("approved", 0.03, 0.01, 0.82, 3.0, 900000.0, Regime::TrendUp),
            evidence_summary: "approved".to_string(),
            owner_context: Some(CommitteeCycleOwnerContext {
                owner_inputs: vec![],
                protocol: HumanConfirmProtocolConfig::default(),
            }),
            risk_snapshot: Some(risk_snapshot()),
            reason_codes: vec![],
        })
        .unwrap();
    assert_eq!(
        approved.candidate_after_status,
        CandidateLifecycleStatus::PaperPositionOpen
    );

    let blocked = CommitteeCycleRunner::default()
        .run_cycle(&CommitteeCycleInput {
            candidate: candidate(
                "blocked",
                0.03,
                0.015,
                0.84,
                18.0,
                100000.0,
                Regime::TrendUp,
            ),
            evidence_summary: "blocked".to_string(),
            owner_context: Some(CommitteeCycleOwnerContext {
                owner_inputs: vec![],
                protocol: HumanConfirmProtocolConfig::default(),
            }),
            risk_snapshot: Some(risk_snapshot()),
            reason_codes: vec![],
        })
        .unwrap();
    assert_eq!(
        blocked.candidate_after_status,
        CandidateLifecycleStatus::RiskBlocked
    );

    let no_trade = CommitteeCycleRunner::default()
        .run_cycle(&CommitteeCycleInput {
            candidate: candidate("notrade", 0.0, 0.03, 0.40, 4.0, 800000.0, Regime::RiskOff),
            evidence_summary: "no trade".to_string(),
            owner_context: Some(CommitteeCycleOwnerContext {
                owner_inputs: vec![],
                protocol: HumanConfirmProtocolConfig::default(),
            }),
            risk_snapshot: Some(risk_snapshot()),
            reason_codes: vec![],
        })
        .unwrap();
    assert_eq!(
        no_trade.candidate_after_status,
        CandidateLifecycleStatus::NoTrade
    );

    let human_confirm = CommitteeCycleRunner {
        enable_paper_position_lifecycle: false,
        ..CommitteeCycleRunner::default()
    }
    .run_cycle(&CommitteeCycleInput {
        candidate: candidate("human", 0.015, 0.015, 0.62, 5.0, 700000.0, Regime::Range),
        evidence_summary: "human".to_string(),
        owner_context: Some(CommitteeCycleOwnerContext {
            owner_inputs: vec![],
            protocol: HumanConfirmProtocolConfig::default(),
        }),
        risk_snapshot: Some(risk_snapshot()),
        reason_codes: vec![],
    })
    .unwrap();
    assert_eq!(
        human_confirm.candidate_after_status,
        CandidateLifecycleStatus::HumanConfirmRequired
    );
    assert!(human_confirm.owner_review_item.is_some());

    let owner_held = CommitteeCycleRunner {
        enable_paper_position_lifecycle: false,
        ..CommitteeCycleRunner::default()
    }
    .run_cycle(&CommitteeCycleInput {
        candidate: candidate(
            "human-hold",
            0.015,
            0.015,
            0.62,
            5.0,
            700000.0,
            Regime::Range,
        ),
        evidence_summary: "hold".to_string(),
        owner_context: Some(CommitteeCycleOwnerContext {
            owner_inputs: vec![owner_input(OwnerInputKind::CandidateHold, "human-hold")],
            protocol: HumanConfirmProtocolConfig::default(),
        }),
        risk_snapshot: Some(risk_snapshot()),
        reason_codes: vec![],
    })
    .unwrap();
    assert_eq!(
        owner_held.candidate_after_status,
        CandidateLifecycleStatus::OwnerHeld
    );

    let owner_dismissed = CommitteeCycleRunner {
        enable_paper_position_lifecycle: false,
        ..CommitteeCycleRunner::default()
    }
    .run_cycle(&CommitteeCycleInput {
        candidate: candidate(
            "human-dismiss",
            0.015,
            0.015,
            0.62,
            5.0,
            700000.0,
            Regime::Range,
        ),
        evidence_summary: "dismiss".to_string(),
        owner_context: Some(CommitteeCycleOwnerContext {
            owner_inputs: vec![owner_input(
                OwnerInputKind::CandidateDismiss,
                "human-dismiss",
            )],
            protocol: HumanConfirmProtocolConfig::default(),
        }),
        risk_snapshot: Some(risk_snapshot()),
        reason_codes: vec![],
    })
    .unwrap();
    assert_eq!(
        owner_dismissed.candidate_after_status,
        CandidateLifecycleStatus::OwnerDismissed
    );

    let reanalysis = CommitteeCycleRunner {
        enable_paper_position_lifecycle: false,
        ..CommitteeCycleRunner::default()
    }
    .run_cycle(&CommitteeCycleInput {
        candidate: candidate(
            "human-reanalyze",
            0.015,
            0.015,
            0.62,
            5.0,
            700000.0,
            Regime::Range,
        ),
        evidence_summary: "reanalyze".to_string(),
        owner_context: Some(CommitteeCycleOwnerContext {
            owner_inputs: vec![owner_input(
                OwnerInputKind::CandidateReanalysisRequest,
                "human-reanalyze",
            )],
            protocol: HumanConfirmProtocolConfig::default(),
        }),
        risk_snapshot: Some(risk_snapshot()),
        reason_codes: vec![],
    })
    .unwrap();
    assert_eq!(
        reanalysis.candidate_after_status,
        CandidateLifecycleStatus::ReanalysisRequested
    );
}
