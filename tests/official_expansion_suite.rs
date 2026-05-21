mod common;
#[path = "support/sprint45_support.rs"]
mod sprint45_support;

use std::collections::BTreeMap;
use std::fs;

use soma_zero::{
    CommitteeScenarioMaterializationLevel, CommitteeScenarioRow, CoreReadinessStatus, MarketVenue,
    OfficialCollectionEntryReport, OfficialCollectionEntryStatus, OfficialCollectionReport,
    OfficialEvidenceExpansionConfig, OfficialEvidenceExpansionRunner,
    OfficialEvidenceExpansionStatus, OfficialReadyRowCompletenessStatus,
    OfficialReadyRowInventoryConfig, OfficialReadyRowInventoryRunner, ProviderKind, ProviderMarket,
    ReasonCode, Regime, StorageBudgetReport, Timeframe, VenueCoverageExpansionPlan,
    VenueCoverageTarget, VenueGroup,
};

fn entry(
    entry_id: &str,
    provider_kind: ProviderKind,
    status: OfficialCollectionEntryStatus,
    ready_for_evidence: bool,
    canonical_csv_path: Option<String>,
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
        canonical_csv_path,
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

fn plan() -> VenueCoverageExpansionPlan {
    VenueCoverageExpansionPlan {
        plan_id: "official-expansion-suite-plan".to_string(),
        targets: vec![VenueCoverageTarget {
            venue_group: VenueGroup::Crypto,
            min_ready_datasets: 1,
            min_outcome_records: 20,
            min_symbols: 1,
            min_timeframes: 1,
            required: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }],
        allow_crypto_only: true,
        allow_missing_equity_auth: true,
        ..VenueCoverageExpansionPlan::default()
    }
}

fn write_report(name: &str, report: &OfficialCollectionReport) -> std::path::PathBuf {
    let dir = common::output_dir(name);
    let path = dir.join("official_collection_report.json");
    fs::write(&path, report.to_json_string().expect("json")).expect("write report");
    path
}

#[test]
fn official_expansion_config_and_inventory_reject_remote_paths() {
    let config = OfficialReadyRowInventoryConfig {
        comparable_evidence_bundle_paths: vec!["https://example.com/bundle.json".to_string()],
        ..OfficialReadyRowInventoryConfig::default()
    };
    assert!(config.validate().is_err());

    let config = OfficialReadyRowInventoryConfig {
        max_rows: 0,
        ..OfficialReadyRowInventoryConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn core_check_failure_maps_to_core_blocked() {
    let report_path = write_report(
        "official-expansion-core-blocked",
        &OfficialCollectionReport {
            plan_id: "core-blocked".to_string(),
            ready_entries_count: 1,
            skipped_entries_count: 0,
            failed_entries_count: 0,
            official_api_collected_count: 1,
            entry_reports: vec![entry(
                "upbit-btc",
                ProviderKind::Upbit,
                OfficialCollectionEntryStatus::Collected,
                true,
                Some(
                    common::fixture_path("generic_ohlcv_valid.csv")
                        .display()
                        .to_string(),
                ),
            )],
            storage_budget_report: StorageBudgetReport::default(),
            reason_codes: vec![ReasonCode::OfficialCollectionRan],
        },
    );
    let report = OfficialEvidenceExpansionRunner::default()
        .run(&OfficialEvidenceExpansionConfig {
            expansion_id: "core-blocked".to_string(),
            venue_coverage_plan: VenueCoverageExpansionPlan {
                existing_collection_report_path: Some(report_path.display().to_string()),
                ..plan()
            },
            allowed_core_statuses: vec![CoreReadinessStatus::NotReadyDueToContractDrift],
            output_root: common::output_dir("official-expansion-core-blocked-out")
                .display()
                .to_string(),
            ..OfficialEvidenceExpansionConfig::default()
        })
        .expect("run");
    assert_eq!(
        report.final_status,
        OfficialEvidenceExpansionStatus::CoreBlocked
    );
    assert_eq!(
        report.nested_benchmark_status.as_deref(),
        Some("CoreBlocked")
    );
}

#[test]
fn storage_preflight_and_benchmark_failures_map_conservatively() {
    let storage_path = write_report(
        "official-expansion-storage-blocked",
        &OfficialCollectionReport {
            plan_id: "storage-blocked".to_string(),
            ready_entries_count: 1,
            skipped_entries_count: 0,
            failed_entries_count: 0,
            official_api_collected_count: 1,
            entry_reports: vec![entry(
                "upbit-btc",
                ProviderKind::Upbit,
                OfficialCollectionEntryStatus::Collected,
                true,
                Some(
                    common::fixture_path("generic_ohlcv_valid.csv")
                        .display()
                        .to_string(),
                ),
            )],
            storage_budget_report: StorageBudgetReport {
                total_bytes: 4096,
                ..StorageBudgetReport::default()
            },
            reason_codes: vec![ReasonCode::OfficialCollectionRan],
        },
    );
    let storage_report = OfficialEvidenceExpansionRunner::default()
        .run(&OfficialEvidenceExpansionConfig {
            expansion_id: "storage-blocked".to_string(),
            venue_coverage_plan: VenueCoverageExpansionPlan {
                existing_collection_report_path: Some(storage_path.display().to_string()),
                ..plan()
            },
            run_core_benchmark: false,
            max_storage_bytes: 1024,
            output_root: common::output_dir("official-expansion-storage-blocked-out")
                .display()
                .to_string(),
            ..OfficialEvidenceExpansionConfig::default()
        })
        .expect("run");
    assert_eq!(
        storage_report.final_status,
        OfficialEvidenceExpansionStatus::StorageBudgetBlocked
    );

    let preflight_path = write_report(
        "official-expansion-preflight-blocked",
        &OfficialCollectionReport {
            plan_id: "preflight-blocked".to_string(),
            ready_entries_count: 0,
            skipped_entries_count: 1,
            failed_entries_count: 1,
            official_api_collected_count: 1,
            entry_reports: vec![entry(
                "upbit-btc",
                ProviderKind::Upbit,
                OfficialCollectionEntryStatus::FailedPreflight,
                false,
                None,
            )],
            storage_budget_report: StorageBudgetReport::default(),
            reason_codes: vec![ReasonCode::OfficialCollectionRan],
        },
    );
    let preflight_report = OfficialEvidenceExpansionRunner::default()
        .run(&OfficialEvidenceExpansionConfig {
            expansion_id: "preflight-blocked".to_string(),
            venue_coverage_plan: VenueCoverageExpansionPlan {
                existing_collection_report_path: Some(preflight_path.display().to_string()),
                ..plan()
            },
            run_core_benchmark: false,
            output_root: common::output_dir("official-expansion-preflight-blocked-out")
                .display()
                .to_string(),
            ..OfficialEvidenceExpansionConfig::default()
        })
        .expect("run");
    assert_eq!(
        preflight_report.final_status,
        OfficialEvidenceExpansionStatus::PreflightBlocked
    );

    let coverage_report = soma_zero::build_venue_coverage_report(
        &VenueCoverageExpansionPlan {
            allow_crypto_only: true,
            ..plan()
        },
        None,
        None,
    );
    let (status, _, blockers, _, _) =
        soma_zero::experiment::official_expansion::classify_official_evidence_expansion_state(
            &OfficialEvidenceExpansionConfig::default(),
            None,
            None,
            &coverage_report,
            None,
            Some("synthetic benchmark failure"),
            &soma_zero::OfficialStorageDelta {
                previous_total_bytes: None,
                current_total_bytes: 0,
                added_bytes: 0,
                added_raw_bytes: 0,
                added_canonical_bytes: 0,
                added_report_bytes: 0,
                budget_exceeded: false,
                largest_new_artifacts: vec![],
                compaction_recommendation: String::new(),
                reason_codes: vec![ReasonCode::OfficialStorageDeltaBuilt],
            },
        );
    assert_eq!(status, OfficialEvidenceExpansionStatus::BenchmarkBlocked);
    assert!(
        blockers
            .iter()
            .any(|blocker| blocker.contains("synthetic benchmark failure"))
    );
}

#[test]
fn official_ready_inventory_preserves_missing_reference_and_source_boundaries() {
    let complete = sprint45_support::row("complete");
    let mut missing = sprint45_support::row("missing");
    missing.outcome_reference_available = false;
    missing.baseline_reference_available = false;
    missing.no_trade_counterfactual_available = false;
    missing.risk_denied_counterfactual_available = false;
    let scenario_rows = BTreeMap::from([(
        "scenario-complete".to_string(),
        CommitteeScenarioRow {
            scenario_row_id: "scenario-complete".to_string(),
            symbol: "AAPL".to_string(),
            timestamp_ms: 1_700_000_000_000,
            source_kind: soma_zero::CommitteeScenarioSourceKind::OfficialBenchmarkReport,
            evidence_source_kind: soma_zero::EvidenceSourceKind::OfficialApiCollected,
            market: ProviderMarket::USEquity,
            target_horizon: soma_zero::PersonaHorizon::Swing,
            feature_vector: None,
            regime: Regime::TrendUp,
            signal_summary: "feature-summary".to_string(),
            data_quality_score: 0.9,
            spread_bps: Some(4.0),
            expected_edge_after_cost: 0.02,
            expected_drawdown: 0.01,
            risk_snapshot_summary: None,
            provenance_summary: "local".to_string(),
            benchmark_status: None,
            baseline_signal_summary: None,
            external_prediction_summary: None,
            no_trade_counterfactual: None,
            risk_denial_counterfactual: None,
            outcome_reference: None,
            materialization_level: CommitteeScenarioMaterializationLevel::RowLevel,
            materialization_confidence: 1.0,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
    )]);
    let report = OfficialReadyRowInventoryRunner::default()
        .run_from_rows(
            &OfficialReadyRowInventoryConfig::default(),
            &[complete, missing],
            &scenario_rows,
            &BTreeMap::new(),
        )
        .expect("inventory");
    assert_eq!(report.official_ready_match_count, 2);
    assert_eq!(report.complete_comparable_row_count, 1);
    assert_eq!(report.missing_outcome_count, 1);
    assert_eq!(report.missing_baseline_count, 1);
    assert_eq!(report.missing_no_trade_count, 1);
    assert_eq!(report.missing_risk_denied_count, 1);

    let mut yfinance = sprint45_support::row("yf");
    yfinance.source_class = soma_zero::ComparableEvidenceSourceClass::YFinanceResearch;
    let mut controlled = sprint45_support::row("controlled");
    controlled.source_class = soma_zero::ComparableEvidenceSourceClass::ControlledDiagnostic;
    let report = OfficialReadyRowInventoryRunner::default()
        .run_from_rows(
            &OfficialReadyRowInventoryConfig::default(),
            &[yfinance, controlled],
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .expect("inventory");
    assert_eq!(report.source_ineligible_count, 2);
    assert!(report.items.iter().all(|item| {
        item.completeness_statuses
            .contains(&OfficialReadyRowCompletenessStatus::SourceIneligible)
    }));
}

#[test]
fn official_expansion_inventory_mapping_is_deterministic() {
    let rows = vec![sprint45_support::row("b"), sprint45_support::row("a")];
    let first = OfficialReadyRowInventoryRunner::default()
        .run_from_rows(
            &OfficialReadyRowInventoryConfig::default(),
            &rows,
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .expect("first");
    let second = OfficialReadyRowInventoryRunner::default()
        .run_from_rows(
            &OfficialReadyRowInventoryConfig::default(),
            &rows,
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .expect("second");
    assert_eq!(first.to_text(), second.to_text());
}
