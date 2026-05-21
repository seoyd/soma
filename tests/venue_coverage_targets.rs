use std::path::PathBuf;

use soma_zero::{
    MarketVenue, OfficialCollectionEntryReport, OfficialCollectionEntryStatus,
    OfficialCollectionReport, ProviderAuthEnvRequirement, ProviderAuthPreflightConfig,
    ProviderAuthPreflightRunner, ProviderKind, ReasonCode, StorageBudgetReport, Timeframe,
    VenueCoverageExpansionPlan, VenueCoverageStatus, VenueCoverageTarget, VenueGroup,
    build_venue_coverage_report,
};

fn example_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name)
}

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
        plan_id: "coverage".to_string(),
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

fn plan() -> VenueCoverageExpansionPlan {
    VenueCoverageExpansionPlan {
        plan_id: "coverage-plan".to_string(),
        targets: vec![
            VenueCoverageTarget {
                venue_group: VenueGroup::Crypto,
                min_ready_datasets: 1,
                min_outcome_records: 20,
                min_symbols: 1,
                min_timeframes: 1,
                required: true,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            VenueCoverageTarget {
                venue_group: VenueGroup::KoreanEquity,
                min_ready_datasets: 1,
                min_outcome_records: 20,
                min_symbols: 1,
                min_timeframes: 1,
                required: true,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            VenueCoverageTarget {
                venue_group: VenueGroup::USEquity,
                min_ready_datasets: 1,
                min_outcome_records: 20,
                min_symbols: 1,
                min_timeframes: 1,
                required: true,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
        ],
        allow_crypto_only: false,
        allow_missing_equity_auth: false,
        ..VenueCoverageExpansionPlan::default()
    }
}

#[test]
fn upbit_only_coverage_is_crypto_only() {
    let report = build_venue_coverage_report(
        &VenueCoverageExpansionPlan {
            allow_crypto_only: true,
            ..plan()
        },
        Some(&report(vec![entry(
            "upbit-btc",
            ProviderKind::Upbit,
            OfficialCollectionEntryStatus::Collected,
            true,
        )])),
        None,
    );

    assert_eq!(report.coverage_status, VenueCoverageStatus::CryptoOnly);
}

#[test]
fn krx_missing_auth_blocks_korean_equity_target() {
    let key = "SOMA_TEST_KRX_KEY_BLOCK";
    let endpoint = "SOMA_TEST_KRX_ENDPOINT_BLOCK";
    unsafe { std::env::remove_var(key) };
    unsafe { std::env::set_var(endpoint, "template") };
    let auth_report = ProviderAuthPreflightRunner::default().run(&ProviderAuthPreflightConfig {
        providers_to_check: vec![ProviderKind::KrxOpenApi],
        required_env_vars: vec![ProviderAuthEnvRequirement {
            provider_kind: ProviderKind::KrxOpenApi,
            api_key_env_var: Some(key.to_string()),
            api_secret_env_var: None,
            endpoint_template_env_var: Some(endpoint.to_string()),
        }],
        ..ProviderAuthPreflightConfig::default()
    });

    let report = build_venue_coverage_report(&plan(), Some(&report(vec![])), Some(&auth_report));
    let korean = report
        .target_results
        .iter()
        .find(|target| target.venue_group == VenueGroup::KoreanEquity)
        .expect("korean target");

    assert!(korean.auth_blocked);
    assert_eq!(report.coverage_status, VenueCoverageStatus::MissingAuth);
    unsafe { std::env::remove_var(endpoint) };
}

#[test]
fn alphavantage_missing_auth_blocks_us_equity_target() {
    let key = "SOMA_TEST_ALPHA_KEY_BLOCK";
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

    let report = build_venue_coverage_report(&plan(), Some(&report(vec![])), Some(&auth_report));
    let us = report
        .target_results
        .iter()
        .find(|target| target.venue_group == VenueGroup::USEquity)
        .expect("us target");

    assert!(us.auth_blocked);
    assert_eq!(report.coverage_status, VenueCoverageStatus::MissingAuth);
}

#[test]
fn mock_entries_are_excluded() {
    let report = build_venue_coverage_report(
        &VenueCoverageExpansionPlan {
            allow_crypto_only: true,
            ..plan()
        },
        Some(&report(vec![entry(
            "mock",
            ProviderKind::MockFixture,
            OfficialCollectionEntryStatus::Collected,
            true,
        )])),
        None,
    );

    assert_eq!(report.coverage_status, VenueCoverageStatus::NoOfficialData);
}

#[test]
fn one_symbol_per_venue_is_marked_weak_evidence() {
    let mut weak_plan = plan();
    weak_plan.allow_crypto_only = true;
    weak_plan.targets[0].min_symbols = 2;
    let report = build_venue_coverage_report(
        &weak_plan,
        Some(&report(vec![entry(
            "upbit-btc",
            ProviderKind::Upbit,
            OfficialCollectionEntryStatus::Collected,
            true,
        )])),
        None,
    );

    assert!(report.target_results.iter().any(|target| {
        target
            .reason_codes
            .contains(&ReasonCode::VenueCoverageWeakEvidence)
    }));
}

#[test]
fn multi_venue_ready_requires_required_targets_to_pass() {
    let report = build_venue_coverage_report(
        &plan(),
        Some(&report(vec![
            entry(
                "upbit-btc",
                ProviderKind::Upbit,
                OfficialCollectionEntryStatus::Collected,
                true,
            ),
            entry(
                "krx-005930",
                ProviderKind::KrxOpenApi,
                OfficialCollectionEntryStatus::Collected,
                true,
            ),
            entry(
                "av-aapl",
                ProviderKind::AlphaVantage,
                OfficialCollectionEntryStatus::Collected,
                true,
            ),
        ])),
        None,
    );

    assert_eq!(report.coverage_status, VenueCoverageStatus::MultiVenueReady);
}

#[test]
fn sprint25_venue_coverage_example_parses() {
    let plan = VenueCoverageExpansionPlan::from_toml_path(&example_path(
        "soma_venue_coverage_targets.toml",
    ))
    .expect("parse coverage example");

    assert_eq!(plan.targets.len(), 3);
    assert!(!plan.allow_crypto_only);
}

#[test]
fn coverage_report_is_deterministic() {
    let collection = report(vec![entry(
        "upbit-btc",
        ProviderKind::Upbit,
        OfficialCollectionEntryStatus::Collected,
        true,
    )]);
    let first = build_venue_coverage_report(
        &VenueCoverageExpansionPlan {
            allow_crypto_only: true,
            ..plan()
        },
        Some(&collection),
        None,
    );
    let second = build_venue_coverage_report(
        &VenueCoverageExpansionPlan {
            allow_crypto_only: true,
            ..plan()
        },
        Some(&collection),
        None,
    );

    assert_eq!(first, second);
}
