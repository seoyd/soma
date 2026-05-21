use soma_zero::{EvidenceSourceKind, SourceDatasetRecord, build_source_overlap_report};

fn record(
    source_kind: EvidenceSourceKind,
    symbol: &str,
    timeframe: &str,
    adjusted: Option<&str>,
) -> SourceDatasetRecord {
    SourceDatasetRecord {
        dataset_id: format!("{source_kind:?}-{symbol}"),
        source_kind,
        symbol: symbol.to_string(),
        normalized_symbol: symbol.to_string(),
        timeframe_label: timeframe.to_string(),
        venue: None,
        canonical_csv_path: None,
        manifest_path: None,
        provenance_path: None,
        row_count: 10,
        ready_for_evidence: source_kind == EvidenceSourceKind::OfficialApiCollected,
        benchmark_eligible: source_kind == EvidenceSourceKind::YFinanceResearch,
        adjusted_price_policy: adjusted.map(|value| value.to_string()),
        data_quality_score: None,
        reason_codes: vec![],
    }
}

#[test]
fn overlap_detects_matching_official_and_yfinance_keys() {
    let report = build_source_overlap_report(
        &[record(
            EvidenceSourceKind::OfficialApiCollected,
            "AAPL",
            "OneDay",
            Some("raw"),
        )],
        &[record(
            EvidenceSourceKind::YFinanceResearch,
            "AAPL",
            "OneDay",
            Some("raw"),
        )],
    );
    assert_eq!(report.overlap_count, 1);
    assert!(report.comparable);
}

#[test]
fn yfinance_only_key_is_marked_missing_official() {
    let report = build_source_overlap_report(
        &[],
        &[record(
            EvidenceSourceKind::YFinanceResearch,
            "AAPL",
            "OneDay",
            Some("raw"),
        )],
    );
    assert_eq!(report.missing_official_for_yfinance.len(), 1);
}

#[test]
fn official_only_key_is_marked_missing_yfinance() {
    let report = build_source_overlap_report(
        &[record(
            EvidenceSourceKind::OfficialApiCollected,
            "AAPL",
            "OneDay",
            Some("raw"),
        )],
        &[],
    );
    assert_eq!(report.missing_yfinance_for_official.len(), 1);
}

#[test]
fn adjusted_price_mismatch_marks_overlap_not_comparable() {
    let report = build_source_overlap_report(
        &[record(
            EvidenceSourceKind::OfficialApiCollected,
            "AAPL",
            "OneDay",
            Some("raw"),
        )],
        &[record(
            EvidenceSourceKind::YFinanceResearch,
            "AAPL",
            "OneDay",
            Some("adjusted"),
        )],
    );
    assert!(!report.comparable);
}
