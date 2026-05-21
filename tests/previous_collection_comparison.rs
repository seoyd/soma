mod common;

use std::fs;

use soma_zero::{
    MarketVenue, OfficialCollectionEntryReport, OfficialCollectionEntryStatus,
    OfficialCollectionReport, ProviderAuthEnvRequirement, ProviderAuthPreflightConfig,
    ProviderAuthPreflightRunner, ProviderKind, ReasonCode, StorageBudgetReport, Timeframe,
    build_previous_collection_comparison, load_previous_collection_report,
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
        symbol: entry_id.to_string(),
        venue: Some(match provider_kind {
            ProviderKind::Upbit => MarketVenue::Upbit,
            ProviderKind::KrxOpenApi => MarketVenue::KRX,
            ProviderKind::AlphaVantage => MarketVenue::US,
            _ => MarketVenue::Generic,
        }),
        timeframe: Timeframe::OneDay,
        status,
        canonical_csv_path: Some(
            common::fixture_path("generic_ohlcv_valid.csv")
                .display()
                .to_string(),
        ),
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

fn report(name: &str, entries: Vec<OfficialCollectionEntryReport>) -> OfficialCollectionReport {
    OfficialCollectionReport {
        plan_id: name.to_string(),
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

#[test]
fn missing_previous_collection_report_is_reason_coded_not_panic() {
    let missing_path = common::output_dir("previous-comparison-missing").join("missing.json");
    let (previous, reasons) =
        load_previous_collection_report(Some(&missing_path.display().to_string())).expect("load");

    let comparison = build_previous_collection_comparison(None, None, None, true, &reasons);

    assert!(previous.is_none());
    assert!(!comparison.comparable);
    assert!(comparison.reason_codes.contains(&ReasonCode::MissingFile));
}

#[test]
fn previous_collection_report_loads_when_file_exists() {
    let path = common::output_dir("previous-comparison-load").join("previous.json");
    let report = report(
        "previous-load",
        vec![entry(
            "upbit-btc",
            ProviderKind::Upbit,
            OfficialCollectionEntryStatus::Collected,
            true,
        )],
    );
    fs::write(&path, report.to_json_string().expect("json")).expect("write report");

    let (loaded, reasons) =
        load_previous_collection_report(Some(&path.display().to_string())).expect("load previous");

    assert!(loaded.is_some());
    assert!(reasons.contains(&ReasonCode::PreviousCollectionReportLoaded));
}

#[test]
fn ready_entry_delta_is_computed() {
    let comparison = build_previous_collection_comparison(
        Some(&report(
            "previous-ready",
            vec![entry(
                "upbit-btc",
                ProviderKind::Upbit,
                OfficialCollectionEntryStatus::Collected,
                true,
            )],
        )),
        Some(&report(
            "current-ready",
            vec![
                entry(
                    "upbit-btc",
                    ProviderKind::Upbit,
                    OfficialCollectionEntryStatus::Collected,
                    true,
                ),
                entry(
                    "us-aapl",
                    ProviderKind::AlphaVantage,
                    OfficialCollectionEntryStatus::Collected,
                    true,
                ),
            ],
        )),
        None,
        true,
        &[],
    );

    assert!(comparison.comparable);
    assert_eq!(comparison.added_ready_entries, vec!["us-aapl".to_string()]);
    assert!(comparison.removed_ready_entries.is_empty());
}

#[test]
fn missing_auth_delta_is_computed() {
    let key = "SOMA_TEST_PREVIOUS_ALPHA_KEY";
    unsafe { std::env::remove_var(key) };
    let auth_report = ProviderAuthPreflightRunner::default().run(&ProviderAuthPreflightConfig {
        providers_to_check: vec![ProviderKind::AlphaVantage],
        required_env_vars: vec![ProviderAuthEnvRequirement {
            provider_kind: ProviderKind::AlphaVantage,
            api_key_env_var: Some(key.to_string()),
            api_secret_env_var: None,
            endpoint_template_env_var: None,
        }],
        ..ProviderAuthPreflightConfig::default()
    });
    let comparison = build_previous_collection_comparison(
        Some(&report(
            "previous-auth",
            vec![entry(
                "krx-005930",
                ProviderKind::KrxOpenApi,
                OfficialCollectionEntryStatus::SkippedMissingAuth,
                false,
            )],
        )),
        Some(&report("current-auth", vec![])),
        Some(&auth_report),
        true,
        &[],
    );

    assert_eq!(comparison.fixed_missing_auth, vec!["krx".to_string()]);
    assert_eq!(
        comparison.new_missing_auth,
        vec!["alphavantage".to_string()]
    );
}

#[test]
fn previous_collection_comparison_is_deterministic() {
    let current = report(
        "det-current",
        vec![entry(
            "upbit-btc",
            ProviderKind::Upbit,
            OfficialCollectionEntryStatus::Collected,
            true,
        )],
    );
    let first = serde_json::to_string(&build_previous_collection_comparison(
        None,
        Some(&current),
        None,
        false,
        &[],
    ))
    .expect("first");
    let second = serde_json::to_string(&build_previous_collection_comparison(
        None,
        Some(&current),
        None,
        false,
        &[],
    ))
    .expect("second");

    assert_eq!(first, second);
}
