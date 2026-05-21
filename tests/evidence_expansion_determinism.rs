mod common;

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    MarketVenue, OfficialCollectionEntryReport, OfficialCollectionEntryStatus,
    OfficialCollectionReport, OfficialEvidenceExpansionConfig, OfficialEvidenceExpansionRunner,
    ProviderKind, ReasonCode, StorageBudgetReport, Timeframe, VenueCoverageExpansionPlan,
};

fn write_collection_report(name: &str) -> PathBuf {
    let output_dir = common::output_dir(name);
    let report_path = output_dir.join("official_collection_report.json");
    let report = OfficialCollectionReport {
        plan_id: name.to_string(),
        entry_reports: vec![OfficialCollectionEntryReport {
            entry_id: "upbit-btc".to_string(),
            provider_kind: ProviderKind::Upbit,
            symbol: "BTC-USDT".to_string(),
            venue: Some(MarketVenue::Upbit),
            timeframe: Timeframe::OneDay,
            status: OfficialCollectionEntryStatus::Collected,
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
            ready_for_evidence: true,
            reason_codes: vec![ReasonCode::OfficialCollectionEntryCollected],
        }],
        storage_budget_report: StorageBudgetReport::default(),
        ready_entries_count: 1,
        skipped_entries_count: 0,
        failed_entries_count: 0,
        official_api_collected_count: 1,
        reason_codes: vec![ReasonCode::OfficialCollectionRan],
    };
    fs::write(
        &report_path,
        report.to_json_string().expect("serialize report"),
    )
    .expect("write report");
    report_path
}

#[test]
fn evidence_expansion_report_is_deterministic() {
    let report_path = write_collection_report("evidence-expansion-determinism");
    let config = OfficialEvidenceExpansionConfig {
        expansion_id: "evidence-expansion-determinism".to_string(),
        venue_coverage_plan: VenueCoverageExpansionPlan {
            existing_collection_report_path: Some(report_path.display().to_string()),
            ..VenueCoverageExpansionPlan::default()
        },
        output_root: common::output_dir("evidence-expansion-determinism-out")
            .display()
            .to_string(),
        ..OfficialEvidenceExpansionConfig::default()
    };
    let runner = OfficialEvidenceExpansionRunner::default();

    let _ = fs::remove_dir_all(config.output_dir());
    let first = runner.run(&config).expect("first report");
    let _ = fs::remove_dir_all(config.output_dir());
    let second = runner.run(&config).expect("second report");

    assert_eq!(
        first.to_json_string().expect("first json"),
        second.to_json_string().expect("second json")
    );
}
