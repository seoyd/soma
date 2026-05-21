use soma_zero::{
    CandidateEvidenceClass, CandidateLifecycleStatus, CommitteeTaskKind, GeneratedCandidate,
    OwnerInput, OwnerInputKind, OwnerInputStatus, OwnerInputTargetType, ProviderMarket,
    build_committee_work_queue,
};

fn candidate(id: &str, status: CandidateLifecycleStatus) -> GeneratedCandidate {
    GeneratedCandidate {
        candidate_id: id.to_string(),
        symbol: id.to_string(),
        market: ProviderMarket::USEquity,
        timeframe: "1d".to_string(),
        horizon_bars: 20,
        source_kind: soma_zero::CandidateSourceKind::KISOfficialEvidence,
        evidence_class: CandidateEvidenceClass::Official,
        initial_status: status,
        expected_edge: Some(0.02),
        expected_drawdown: Some(0.01),
        data_quality_score: Some(0.9),
        signal_summary: Some("candidate".to_string()),
        timestamp_ms: 1,
        confidence: Some(0.8),
        spread_bps: Some(3.0),
        trade_value: Some(500000.0),
        regime: Some(soma_zero::Regime::TrendUp),
        paper_outcome_hint: None,
        source_report_path: None,
        reason_codes: vec![],
    }
}

#[test]
fn committee_work_queue_is_deterministic_and_trinity_only() {
    let queue = build_committee_work_queue(
        &[
            candidate("a", CandidateLifecycleStatus::EvidenceReady),
            candidate("b", CandidateLifecycleStatus::RiskBlocked),
        ],
        &[OwnerInput {
            owner_input_id: "reanalysis-a".to_string(),
            timestamp_ms: Some(1),
            owner_id: None,
            input_kind: OwnerInputKind::CandidateReanalysisRequest,
            target_type: OwnerInputTargetType::Candidate,
            target_id: Some("a".to_string()),
            symbol: None,
            market: None,
            freeform_note: None,
            structured_payload: None,
            requested_action: None,
            status: OwnerInputStatus::Submitted,
            reason_codes: vec![],
        }],
    );
    assert!(
        queue
            .pending_items
            .iter()
            .any(|item| item.task_kind == CommitteeTaskKind::AnalyzeCandidate)
    );
    assert!(
        queue
            .pending_items
            .iter()
            .any(|item| item.task_kind == CommitteeTaskKind::VoteCandidate)
    );
    assert!(
        queue
            .pending_items
            .iter()
            .any(|item| item.task_kind == CommitteeTaskKind::ReanalyzeCandidate)
    );
    assert!(
        queue
            .blocked_items
            .iter()
            .any(|item| item.task_kind == CommitteeTaskKind::ReviewRiskBlocked)
    );
    assert!(
        queue
            .pending_items
            .iter()
            .all(|item| item.assigned_personas.len() <= 3)
    );
    assert_eq!(
        queue.fingerprint,
        build_committee_work_queue(
            &[
                candidate("a", CandidateLifecycleStatus::EvidenceReady),
                candidate("b", CandidateLifecycleStatus::RiskBlocked)
            ],
            &[OwnerInput {
                owner_input_id: "reanalysis-a".to_string(),
                timestamp_ms: Some(1),
                owner_id: None,
                input_kind: OwnerInputKind::CandidateReanalysisRequest,
                target_type: OwnerInputTargetType::Candidate,
                target_id: Some("a".to_string()),
                symbol: None,
                market: None,
                freeform_note: None,
                structured_payload: None,
                requested_action: None,
                status: OwnerInputStatus::Submitted,
                reason_codes: vec![],
            }]
        )
        .fingerprint
    );
}
