mod common;

use std::fs;

use soma_zero::{BenchmarkStorageAudit, OfficialCollectionReport, StorageBudgetReport};

#[test]
fn storage_audit_counts_bytes_by_output_type() {
    let bundle_dir = common::output_dir("benchmark-storage-bundle");
    fs::write(bundle_dir.join("dataset.csv"), "a,b\n1,2\n").expect("write dataset");
    fs::write(bundle_dir.join("predictions.csv"), "a,b\n1,2\n").expect("write predictions");
    fs::write(bundle_dir.join("summary.txt"), "summary").expect("write summary");

    let report = BenchmarkStorageAudit::build(
        &OfficialCollectionReport {
            plan_id: "storage".to_string(),
            entry_reports: Vec::new(),
            storage_budget_report: StorageBudgetReport {
                total_bytes: 100,
                raw_bytes: 40,
                canonical_bytes: 50,
                manifest_bytes: 10,
                compressed_bytes: 0,
                uncompressed_bytes_estimate: 100,
                file_count: 3,
                budget_exceeded: false,
                compression_applied: false,
                retention_actions: Vec::new(),
                skipped_files: Vec::new(),
                reason_codes: vec![],
            },
            ready_entries_count: 0,
            skipped_entries_count: 0,
            failed_entries_count: 0,
            official_api_collected_count: 0,
            reason_codes: vec![],
        },
        std::slice::from_ref(&bundle_dir),
        &bundle_dir,
    );

    assert!(report.dataset_export_bytes > 0);
    assert!(report.prediction_bytes > 0);
    assert!(report.report_bytes > 0);
    assert!(!report.budget_exceeded);
}

#[test]
fn storage_audit_reason_codes_budget_exceeded() {
    let root = common::output_dir("benchmark-storage-budget");
    let report = BenchmarkStorageAudit::build(
        &OfficialCollectionReport {
            plan_id: "storage".to_string(),
            entry_reports: Vec::new(),
            storage_budget_report: StorageBudgetReport {
                budget_exceeded: true,
                ..StorageBudgetReport::default()
            },
            ready_entries_count: 0,
            skipped_entries_count: 0,
            failed_entries_count: 0,
            official_api_collected_count: 0,
            reason_codes: vec![],
        },
        &[],
        &root,
    );
    assert!(report.budget_exceeded);
    assert!(
        report
            .reason_codes
            .contains(&soma_zero::ReasonCode::CollectionBudgetExceeded)
    );
}
