use soma_zero::{
    OperationalAuditEvent, OperationalAuditTimeline, OperationalEventKind, ReasonCode,
};

#[test]
fn operational_audit_timeline_is_sorted_and_deterministic() {
    let timeline = OperationalAuditTimeline::from_events(vec![
        OperationalAuditEvent::new(
            OperationalEventKind::RiskReviewed,
            Some("c1".to_string()),
            None,
            Some(20),
            Some("ChairReviewed".to_string()),
            Some("RiskReview".to_string()),
            "risk reviewed",
            vec![ReasonCode::DeterministicPath],
        ),
        OperationalAuditEvent::new(
            OperationalEventKind::CandidateGenerated,
            Some("c1".to_string()),
            None,
            Some(10),
            Some("Detected".to_string()),
            Some("EvidenceReady".to_string()),
            "candidate generated",
            vec![ReasonCode::DeterministicPath],
        ),
        OperationalAuditEvent::new(
            OperationalEventKind::PersonaVoted,
            Some("c1".to_string()),
            Some("trend_breakout_fast".to_string()),
            Some(15),
            Some("Voting".to_string()),
            Some("Approve".to_string()),
            "persona voted",
            vec![ReasonCode::DeterministicPath],
        ),
    ]);
    assert_eq!(
        timeline.events.first().unwrap().event_kind,
        OperationalEventKind::CandidateGenerated
    );
    assert_eq!(
        timeline.events.last().unwrap().event_kind,
        OperationalEventKind::RiskReviewed
    );
    assert_eq!(
        timeline.fingerprint,
        OperationalAuditTimeline::from_events(timeline.events.clone()).fingerprint
    );
}
