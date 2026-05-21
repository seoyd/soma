use soma_zero::{
    MarketVenue, OfficialCollectionEntryReport, OfficialCollectionEntryStatus,
    OfficialCollectionReport, OfficialDatasetCoverageReport, ProviderKind, StorageBudgetReport,
    Timeframe,
};

fn entry(
    entry_id: &str,
    provider_kind: ProviderKind,
    venue: Option<MarketVenue>,
    status: OfficialCollectionEntryStatus,
    ready_for_evidence: bool,
) -> OfficialCollectionEntryReport {
    OfficialCollectionEntryReport {
        entry_id: entry_id.to_string(),
        provider_kind,
        symbol: "TEST".to_string(),
        venue,
        timeframe: Timeframe::OneMinute,
        status,
        canonical_csv_path: ready_for_evidence.then_some(format!("target/{entry_id}_compact.csv")),
        manifest_path: None,
        provenance_path: None,
        preflight_status: ready_for_evidence.then_some("ReadyForRealEvidence".to_string()),
        row_count: if ready_for_evidence { 100 } else { 0 },
        request_count: 1,
        bytes_written: 100,
        compressed: false,
        ready_for_evidence,
        reason_codes: vec![soma_zero::ReasonCode::DeterministicPath],
    }
}

fn report(entries: Vec<OfficialCollectionEntryReport>) -> OfficialCollectionReport {
    OfficialCollectionReport {
        plan_id: "coverage".to_string(),
        ready_entries_count: entries
            .iter()
            .filter(|entry| entry.ready_for_evidence)
            .count(),
        skipped_entries_count: entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry.status,
                    OfficialCollectionEntryStatus::SkippedMissingAuth
                        | OfficialCollectionEntryStatus::SkippedBudgetExceeded
                )
            })
            .count(),
        failed_entries_count: entries
            .iter()
            .filter(|entry| matches!(entry.status, OfficialCollectionEntryStatus::FailedProvider))
            .count(),
        official_api_collected_count: entries
            .iter()
            .filter(|entry| {
                entry.provider_kind != ProviderKind::MockFixture && entry.ready_for_evidence
            })
            .count(),
        entry_reports: entries,
        storage_budget_report: StorageBudgetReport::default(),
        reason_codes: vec![soma_zero::ReasonCode::OfficialCollectionRan],
    }
}

#[test]
fn upbit_only_coverage_reports_crypto_only() {
    let coverage = OfficialDatasetCoverageReport::from_collection_report(&report(vec![entry(
        "upbit",
        ProviderKind::Upbit,
        Some(MarketVenue::Upbit),
        OfficialCollectionEntryStatus::Collected,
        true,
    )]));
    assert_eq!(coverage.total_ready_entries, 1);
    assert_eq!(coverage.crypto_ready_entries, 1);
    assert_eq!(coverage.korean_equity_ready_entries, 0);
    assert_eq!(coverage.us_equity_ready_entries, 0);
}

#[test]
fn missing_krx_auth_prevents_korean_equity_readiness_claim() {
    let coverage = OfficialDatasetCoverageReport::from_collection_report(&report(vec![entry(
        "krx",
        ProviderKind::KrxOpenApi,
        Some(MarketVenue::KOSPI),
        OfficialCollectionEntryStatus::SkippedMissingAuth,
        false,
    )]));
    assert_eq!(coverage.korean_equity_ready_entries, 0);
    assert!(coverage.missing_auth_providers.contains(&"krx".to_string()));
}

#[test]
fn mock_fixture_entries_do_not_count_as_official_readiness() {
    let coverage = OfficialDatasetCoverageReport::from_collection_report(&report(vec![entry(
        "mock",
        ProviderKind::MockFixture,
        Some(MarketVenue::NASDAQ),
        OfficialCollectionEntryStatus::Collected,
        true,
    )]));
    assert_eq!(coverage.total_ready_entries, 0);
    assert_eq!(coverage.non_official_ready_entries, 1);
}
