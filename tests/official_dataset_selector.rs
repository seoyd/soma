use soma_zero::{
    MarketVenue, OfficialBenchmarkDatasetSelector, OfficialCollectionEntryReport,
    OfficialCollectionEntryStatus, OfficialCollectionReport, OfficialDatasetCoverageStatus,
    OfficialDatasetSelectionPolicy, ProviderKind, ReasonCode, StorageBudgetReport, Timeframe,
};

fn entry(
    entry_id: &str,
    provider_kind: ProviderKind,
    status: OfficialCollectionEntryStatus,
    ready_for_evidence: bool,
) -> OfficialCollectionEntryReport {
    OfficialCollectionEntryReport {
        entry_id: entry_id.to_string(),
        provider_kind,
        symbol: "BTC-USDT".to_string(),
        venue: Some(match provider_kind {
            ProviderKind::Upbit => MarketVenue::Upbit,
            _ => MarketVenue::Generic,
        }),
        timeframe: Timeframe::OneMinute,
        status,
        canonical_csv_path: None,
        manifest_path: None,
        provenance_path: None,
        preflight_status: Some("ReadyForRealEvidence".to_string()),
        row_count: 120,
        request_count: 1,
        bytes_written: 1024,
        compressed: false,
        ready_for_evidence,
        reason_codes: vec![ReasonCode::OfficialCollectionEntryCollected],
    }
}

fn report(entries: Vec<OfficialCollectionEntryReport>) -> OfficialCollectionReport {
    OfficialCollectionReport {
        plan_id: "selector".to_string(),
        ready_entries_count: entries
            .iter()
            .filter(|entry| entry.ready_for_evidence)
            .count(),
        skipped_entries_count: entries
            .iter()
            .filter(|entry| entry.status != OfficialCollectionEntryStatus::Collected)
            .count(),
        failed_entries_count: 0,
        official_api_collected_count: entries
            .iter()
            .filter(|entry| entry.provider_kind != ProviderKind::MockFixture)
            .count(),
        entry_reports: entries,
        storage_budget_report: StorageBudgetReport::default(),
        reason_codes: vec![ReasonCode::OfficialCollectionRan],
    }
}

fn policy() -> OfficialDatasetSelectionPolicy {
    OfficialDatasetSelectionPolicy {
        min_ready_official_datasets: 1,
        allow_crypto_only: true,
        allow_missing_equity_auth: false,
    }
}

#[test]
fn selector_picks_ready_official_entries_and_excludes_mock() {
    let selection = OfficialBenchmarkDatasetSelector::default().select_ready_entries(
        &report(vec![
            entry(
                "upbit-ready",
                ProviderKind::Upbit,
                OfficialCollectionEntryStatus::Collected,
                true,
            ),
            entry(
                "mock-ready",
                ProviderKind::MockFixture,
                OfficialCollectionEntryStatus::Collected,
                true,
            ),
        ]),
        &policy(),
    );

    assert_eq!(selection.selected_entries, vec!["upbit-ready".to_string()]);
    assert!(
        selection
            .skipped_entries
            .contains(&"mock-ready".to_string())
    );
}

#[test]
fn selector_marks_upbit_only_coverage_as_crypto_only() {
    let selection = OfficialBenchmarkDatasetSelector::default().select_ready_entries(
        &report(vec![entry(
            "upbit-ready",
            ProviderKind::Upbit,
            OfficialCollectionEntryStatus::Collected,
            true,
        )]),
        &policy(),
    );

    assert_eq!(
        selection.coverage_status,
        OfficialDatasetCoverageStatus::CryptoOnly
    );
    assert!(
        selection
            .reason_codes
            .contains(&ReasonCode::BenchmarkCryptoOnlyEvidence)
    );
}

#[test]
fn selector_tracks_missing_equity_auth_conservatively() {
    let selection = OfficialBenchmarkDatasetSelector::default().select_ready_entries(
        &report(vec![
            entry(
                "upbit-ready",
                ProviderKind::Upbit,
                OfficialCollectionEntryStatus::Collected,
                true,
            ),
            entry(
                "krx-auth-missing",
                ProviderKind::KrxOpenApi,
                OfficialCollectionEntryStatus::SkippedMissingAuth,
                false,
            ),
            entry(
                "alpha-auth-missing",
                ProviderKind::AlphaVantage,
                OfficialCollectionEntryStatus::SkippedMissingAuth,
                false,
            ),
        ]),
        &policy(),
    );

    assert_eq!(
        selection.coverage_status,
        OfficialDatasetCoverageStatus::MissingEquityAuth
    );
    assert_eq!(selection.missing_auth_entries.len(), 2);
    assert!(selection.reason_codes.contains(&ReasonCode::MissingAuth));
}

#[test]
fn selector_is_deterministic_for_same_input() {
    let report = report(vec![
        entry(
            "upbit-ready",
            ProviderKind::Upbit,
            OfficialCollectionEntryStatus::Collected,
            true,
        ),
        entry(
            "krx-auth-missing",
            ProviderKind::KrxOpenApi,
            OfficialCollectionEntryStatus::SkippedMissingAuth,
            false,
        ),
    ]);
    let selector = OfficialBenchmarkDatasetSelector::default();

    let first = selector.select_ready_entries(&report, &policy());
    let second = selector.select_ready_entries(&report, &policy());

    assert_eq!(first, second);
}
