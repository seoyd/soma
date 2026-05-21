use soma_zero::{OwnerThesisBook, OwnerThesisNote, OwnerThesisType};

#[test]
fn owner_thesis_book_groups_active_notes_and_excludes_expired() {
    let notes = vec![
        OwnerThesisNote {
            thesis_id: "thesis-active".to_string(),
            symbol: Some("005930.KS".to_string()),
            market: Some("KoreanEquity".to_string()),
            timeframe: Some("1d".to_string()),
            thesis_type: OwnerThesisType::Bullish,
            text: "active".to_string(),
            structured_tags: vec!["tag-a".to_string()],
            evidence_links: None,
            expires_at_timestamp_ms: Some(20),
            active: true,
            reason_codes: vec![],
        },
        OwnerThesisNote {
            thesis_id: "thesis-expired".to_string(),
            symbol: Some("AAPL".to_string()),
            market: Some("USEquity".to_string()),
            timeframe: Some("1d".to_string()),
            thesis_type: OwnerThesisType::EventNote,
            text: "expired".to_string(),
            structured_tags: vec!["tag-b".to_string()],
            evidence_links: None,
            expires_at_timestamp_ms: Some(10),
            active: true,
            reason_codes: vec![],
        },
    ];
    let book = OwnerThesisBook::from_notes(&notes, Some(15));
    assert_eq!(book.active_notes.len(), 1);
    assert_eq!(book.expired_notes.len(), 1);
    assert_eq!(book.notes_by_symbol.get("005930.KS").map(Vec::len), Some(1));
    assert!(!book.active_notes[0].is_signal());
    assert_eq!(
        book.fingerprint(),
        OwnerThesisBook::from_notes(&notes, Some(15)).fingerprint()
    );
}
