mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    CoreReadinessStatus, MarketVenue, OfficialCollectionEntryReport, OfficialCollectionEntryStatus,
    OfficialCollectionReport, OfficialEvidenceExpansionConfig, OfficialEvidenceExpansionRunner,
    OfficialEvidenceExpansionStatus, ProviderAuthEnvRequirement, ProviderAuthPreflightConfig,
    ProviderKind, ReasonCode, StorageBudgetReport, Timeframe, VenueCoverageExpansionPlan,
    VenueCoverageTarget, VenueGroup,
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

fn write_collection_report(name: &str, entries: Vec<OfficialCollectionEntryReport>) -> PathBuf {
    let output_dir = common::output_dir(name);
    let report_path = output_dir.join("official_collection_report.json");
    let report = OfficialCollectionReport {
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
    };
    fs::write(
        &report_path,
        report.to_json_string().expect("serialize report"),
    )
    .expect("write report");
    report_path
}

fn plan() -> VenueCoverageExpansionPlan {
    VenueCoverageExpansionPlan {
        plan_id: "expansion-plan".to_string(),
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
                required: false,
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
        ],
        allow_crypto_only: true,
        allow_missing_equity_auth: true,
        ..VenueCoverageExpansionPlan::default()
    }
}

#[test]
fn auth_preflight_failure_is_reason_coded() {
    unsafe { std::env::set_var("SOMA_TEST_KRX_ENDPOINT_PRESENT", "template") };
    let report = OfficialEvidenceExpansionRunner::default()
        .run(&OfficialEvidenceExpansionConfig {
            expansion_id: "expansion-auth-missing".to_string(),
            auth_preflight_config: Some(ProviderAuthPreflightConfig {
                providers_to_check: vec![ProviderKind::KrxOpenApi],
                required_env_vars: vec![ProviderAuthEnvRequirement {
                    provider_kind: ProviderKind::KrxOpenApi,
                    api_key_env_var: Some("SOMA_TEST_KRX_AUTH_MISSING".to_string()),
                    api_secret_env_var: None,
                    endpoint_template_env_var: Some("SOMA_TEST_KRX_ENDPOINT_PRESENT".to_string()),
                }],
                fail_on_missing_required_auth: true,
                allow_missing_optional_auth: false,
                ..ProviderAuthPreflightConfig::default()
            }),
            venue_coverage_plan: VenueCoverageExpansionPlan {
                allow_missing_equity_auth: false,
                ..plan()
            },
            run_auth_preflight: true,
            run_core_benchmark: false,
            output_root: common::output_dir("expansion-auth-missing")
                .display()
                .to_string(),
            ..OfficialEvidenceExpansionConfig::default()
        })
        .expect("run expansion");

    assert_eq!(
        report.final_status,
        OfficialEvidenceExpansionStatus::MissingAuth
    );
    assert!(
        report
            .auth_preflight_report
            .expect("auth report")
            .missing_auth_providers
            .contains(&"krx".to_string())
    );
    unsafe { std::env::remove_var("SOMA_TEST_KRX_ENDPOINT_PRESENT") };
}

#[test]
fn missing_auth_entries_are_skipped_if_configured() {
    let report_path = write_collection_report(
        "expansion-skipped-auth",
        vec![
            entry(
                "upbit-btc",
                ProviderKind::Upbit,
                OfficialCollectionEntryStatus::Collected,
                true,
            ),
            entry(
                "krx-005930",
                ProviderKind::KrxOpenApi,
                OfficialCollectionEntryStatus::SkippedMissingAuth,
                false,
            ),
        ],
    );

    let report = OfficialEvidenceExpansionRunner::default()
        .run(&OfficialEvidenceExpansionConfig {
            expansion_id: "expansion-skipped-auth".to_string(),
            venue_coverage_plan: VenueCoverageExpansionPlan {
                existing_collection_report_path: Some(report_path.display().to_string()),
                ..plan()
            },
            run_core_benchmark: false,
            output_root: common::output_dir("expansion-skipped-auth-out")
                .display()
                .to_string(),
            ..OfficialEvidenceExpansionConfig::default()
        })
        .expect("run expansion");

    assert!(
        report
            .venue_coverage_report
            .missing_auth_summary
            .iter()
            .any(|provider| provider == "krx")
    );
}

#[test]
fn collection_report_is_loaded_and_ready_entries_are_selected() {
    let report_path = write_collection_report(
        "expansion-load-report",
        vec![entry(
            "upbit-btc",
            ProviderKind::Upbit,
            OfficialCollectionEntryStatus::Collected,
            true,
        )],
    );

    let report = OfficialEvidenceExpansionRunner::default()
        .run(&OfficialEvidenceExpansionConfig {
            expansion_id: "expansion-load-report".to_string(),
            venue_coverage_plan: VenueCoverageExpansionPlan {
                existing_collection_report_path: Some(report_path.display().to_string()),
                ..plan()
            },
            run_core_benchmark: true,
            output_root: common::output_dir("expansion-load-report-out")
                .display()
                .to_string(),
            ..OfficialEvidenceExpansionConfig::default()
        })
        .expect("run expansion");

    assert!(report.collection_report.is_some());
    assert!(
        report
            .core_benchmark_report
            .expect("core benchmark")
            .dataset_selection
            .expect("dataset selection")
            .selected_entries
            .contains(&"upbit-btc".to_string())
    );
}

#[test]
fn no_ready_entries_produces_missing_official_data() {
    let report_path = write_collection_report("expansion-no-ready", vec![]);
    let report = OfficialEvidenceExpansionRunner::default()
        .run(&OfficialEvidenceExpansionConfig {
            expansion_id: "expansion-no-ready".to_string(),
            venue_coverage_plan: VenueCoverageExpansionPlan {
                existing_collection_report_path: Some(report_path.display().to_string()),
                ..plan()
            },
            run_auth_preflight: false,
            run_core_benchmark: false,
            output_root: common::output_dir("expansion-no-ready-out")
                .display()
                .to_string(),
            ..OfficialEvidenceExpansionConfig::default()
        })
        .expect("run expansion");

    assert_eq!(
        report.final_status,
        OfficialEvidenceExpansionStatus::MissingOfficialData
    );
}

#[test]
fn only_crypto_entries_remain_crypto_only() {
    let report_path = write_collection_report(
        "expansion-crypto-only",
        vec![entry(
            "upbit-btc",
            ProviderKind::Upbit,
            OfficialCollectionEntryStatus::Collected,
            true,
        )],
    );
    let report = OfficialEvidenceExpansionRunner::default()
        .run(&OfficialEvidenceExpansionConfig {
            expansion_id: "expansion-crypto-only".to_string(),
            venue_coverage_plan: VenueCoverageExpansionPlan {
                existing_collection_report_path: Some(report_path.display().to_string()),
                ..plan()
            },
            run_core_benchmark: false,
            output_root: common::output_dir("expansion-crypto-only-out")
                .display()
                .to_string(),
            ..OfficialEvidenceExpansionConfig::default()
        })
        .expect("run expansion");

    assert_eq!(
        report.final_status,
        OfficialEvidenceExpansionStatus::CryptoOnly
    );
}

#[test]
fn core_check_failure_blocks_benchmark_inside_expansion_runner() {
    let report_path = write_collection_report(
        "expansion-core-blocked",
        vec![entry(
            "upbit-btc",
            ProviderKind::Upbit,
            OfficialCollectionEntryStatus::Collected,
            true,
        )],
    );
    let report = OfficialEvidenceExpansionRunner::default()
        .run(&OfficialEvidenceExpansionConfig {
            expansion_id: "expansion-core-blocked".to_string(),
            venue_coverage_plan: VenueCoverageExpansionPlan {
                existing_collection_report_path: Some(report_path.display().to_string()),
                ..plan()
            },
            allowed_core_statuses: vec![CoreReadinessStatus::NotReadyDueToContractDrift],
            output_root: common::output_dir("expansion-core-blocked-out")
                .display()
                .to_string(),
            ..OfficialEvidenceExpansionConfig::default()
        })
        .expect("run expansion");

    assert_eq!(
        report
            .core_benchmark_report
            .expect("core benchmark report")
            .final_status,
        soma_zero::CoreCheckedBenchmarkStatus::CoreBlocked
    );
}

#[test]
fn baseline_only_path_works_without_python_and_external_eval_can_be_skipped() {
    let report_path = write_collection_report(
        "expansion-baseline-only",
        vec![entry(
            "upbit-btc",
            ProviderKind::Upbit,
            OfficialCollectionEntryStatus::Collected,
            true,
        )],
    );
    let report = OfficialEvidenceExpansionRunner::default()
        .run(&OfficialEvidenceExpansionConfig {
            expansion_id: "expansion-baseline-only".to_string(),
            venue_coverage_plan: VenueCoverageExpansionPlan {
                existing_collection_report_path: Some(report_path.display().to_string()),
                ..plan()
            },
            run_external_eval: false,
            output_root: common::output_dir("expansion-baseline-only-out")
                .display()
                .to_string(),
            ..OfficialEvidenceExpansionConfig::default()
        })
        .expect("run expansion");
    let benchmark = report.core_benchmark_report.expect("core benchmark");

    assert!(benchmark.baseline_report.is_some());
    assert!(benchmark.external_report.is_none());
}
