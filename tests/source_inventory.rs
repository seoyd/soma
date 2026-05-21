use soma_zero::{EvidenceSourceKind, SourceDatasetRecord, build_source_kind_dataset_inventory};

fn record(
    id: &str,
    source_kind: EvidenceSourceKind,
    symbol: &str,
    timeframe: &str,
) -> SourceDatasetRecord {
    SourceDatasetRecord {
        dataset_id: id.to_string(),
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
        adjusted_price_policy: None,
        data_quality_score: None,
        reason_codes: vec![],
    }
}

#[test]
fn source_inventory_counts_source_kinds_conservatively() {
    let inventory = build_source_kind_dataset_inventory(&[
        record(
            "official",
            EvidenceSourceKind::OfficialApiCollected,
            "AAPL",
            "OneDay",
        ),
        record(
            "yfinance",
            EvidenceSourceKind::YFinanceResearch,
            "AAPL",
            "OneDay",
        ),
        record(
            "fixture",
            EvidenceSourceKind::TestFixture,
            "BTCUSDT",
            "OneMinute",
        ),
        record(
            "synthetic",
            EvidenceSourceKind::SyntheticFixture,
            "ETHUSDT",
            "OneMinute",
        ),
    ]);

    assert_eq!(inventory.official_ready_count, 1);
    assert_eq!(inventory.yfinance_benchmark_eligible_count, 1);
    assert_eq!(inventory.readiness_eligible_count, 1);
    assert_eq!(inventory.research_only_count, 3);
}

#[test]
fn source_inventory_is_deterministic_by_symbol_and_timeframe() {
    let inventory = build_source_kind_dataset_inventory(&[
        record("a", EvidenceSourceKind::YFinanceResearch, "AAPL", "OneDay"),
        record(
            "b",
            EvidenceSourceKind::OfficialApiCollected,
            "AAPL",
            "OneDay",
        ),
        record(
            "c",
            EvidenceSourceKind::OfficialApiCollected,
            "MSFT",
            "OneDay",
        ),
    ]);
    assert_eq!(inventory.by_symbol.get("AAPL"), Some(&2));
    assert_eq!(inventory.by_timeframe.get("OneDay"), Some(&3));
}
