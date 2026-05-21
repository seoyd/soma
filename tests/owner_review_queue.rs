mod common;

use soma_zero::{
    AllowedOwnerAction, CandidatePanel, CandidateStatus, CandidateView, HumanConfirmProtocolConfig,
    OwnerInput, OwnerInputKind, OwnerInputStatus, OwnerInputTargetType, OwnerReviewItemStatus,
    build_owner_review_queue,
};

fn input(kind: OwnerInputKind, target_id: &str, symbol: &str) -> OwnerInput {
    OwnerInput {
        owner_input_id: format!("{target_id}-{kind:?}"),
        timestamp_ms: Some(1715693000000),
        owner_id: Some("owner-local".to_string()),
        input_kind: kind,
        target_type: OwnerInputTargetType::Candidate,
        target_id: Some(target_id.to_string()),
        symbol: Some(symbol.to_string()),
        market: Some("USEquity".to_string()),
        freeform_note: None,
        structured_payload: None,
        requested_action: Some("review".to_string()),
        status: OwnerInputStatus::Submitted,
        reason_codes: vec![],
    }
}

fn candidate(
    candidate_id: &str,
    symbol: &str,
    status: CandidateStatus,
    source_kind: &str,
) -> CandidateView {
    CandidateView {
        candidate_id: candidate_id.to_string(),
        symbol: symbol.to_string(),
        market: "USEquity".to_string(),
        source_kind: source_kind.to_string(),
        provider_kind: "KIS".to_string(),
        timeframe: "1d".to_string(),
        horizon_bars: 5,
        status,
        signal_summary: "signal".to_string(),
        committee_summary: "committee".to_string(),
        chair_summary: "chair".to_string(),
        risk_summary: "risk".to_string(),
        expected_edge: Some(0.01),
        expected_drawdown: Some(0.01),
        data_quality_score: Some(0.95),
        created_from_report: None,
        expires_at: None,
        owner_feedback_history: Vec::new(),
        owner_hold_active: false,
        owner_dismissed: false,
        owner_reanalysis_requested: false,
        owner_paper_confirmed: false,
        linked_thesis_notes: Vec::new(),
        reason_codes: Vec::new(),
    }
}

#[test]
fn owner_review_queue_covers_pending_blocked_and_research_cases() {
    let panel = CandidatePanel {
        candidates: vec![
            candidate(
                "cand-1",
                "AAA",
                CandidateStatus::HumanConfirmRequired,
                "OfficialNonCrypto",
            ),
            candidate(
                "cand-2",
                "BBB",
                CandidateStatus::RiskBlocked,
                "OfficialNonCrypto",
            ),
            candidate("cand-3", "CCC", CandidateStatus::NoTrade, "ResearchOnly"),
            candidate(
                "cand-4",
                "DDD",
                CandidateStatus::DiagnosticOnly,
                "FixtureOnly",
            ),
        ],
        ..Default::default()
    };
    let protocol = HumanConfirmProtocolConfig::default();
    let queue = build_owner_review_queue(
        "queue-test",
        &panel,
        &[
            input(OwnerInputKind::PaperConfirm, "cand-1", "AAA"),
            input(OwnerInputKind::CandidateDismiss, "cand-3", "CCC"),
        ],
        &protocol,
    );

    assert!(
        queue
            .paper_confirmed_items
            .iter()
            .any(|item| item.candidate_id.as_deref() == Some("cand-1"))
    );
    assert!(queue.blocked_items.iter().all(|item| {
        !item
            .allowed_owner_actions
            .contains(&AllowedOwnerAction::PaperConfirm)
    }));
    assert!(
        queue
            .dismissed_items
            .iter()
            .any(|item| item.candidate_id.as_deref() == Some("cand-3"))
    );
    assert!(
        queue
            .deferred_items
            .iter()
            .any(|item| item.current_status == OwnerReviewItemStatus::DiagnosticOnly)
    );
}

#[test]
fn no_trade_item_can_be_reviewed_and_dismissed_and_queue_is_deterministic() {
    let panel = CandidatePanel {
        candidates: vec![candidate(
            "cand-9",
            "ZZZ",
            CandidateStatus::NoTrade,
            "OfficialNonCrypto",
        )],
        ..Default::default()
    };
    let queue = build_owner_review_queue(
        "queue-test",
        &panel,
        &[input(OwnerInputKind::MarkReviewed, "cand-9", "ZZZ")],
        &HumanConfirmProtocolConfig::default(),
    );
    let item = queue
        .reviewed_items
        .iter()
        .find(|item| item.candidate_id.as_deref() == Some("cand-9"))
        .expect("reviewed item");
    assert!(
        item.allowed_owner_actions
            .contains(&AllowedOwnerAction::Dismiss)
    );
    let second = build_owner_review_queue(
        "queue-test",
        &panel,
        &[input(OwnerInputKind::MarkReviewed, "cand-9", "ZZZ")],
        &HumanConfirmProtocolConfig::default(),
    );
    assert_eq!(queue.fingerprint(), second.fingerprint());
}
