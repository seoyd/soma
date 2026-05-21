mod common;

use std::fs;

use soma_zero::{
    BenchmarkStorageAudit, CoreCheckGateResult, CoreCheckedBenchmarkRecommendation,
    CoreCheckedBenchmarkReport, CoreCheckedBenchmarkStatus, ModelUsefulnessGateResult,
    OfficialCollectionReport, OfficialStorageDelta, PerformanceSummary, ReasonCode,
    StorageBudgetReport, build_official_storage_delta,
};

fn core_report(
    total: usize,
    raw: usize,
    canonical: usize,
    report: usize,
    largest: &[&str],
) -> CoreCheckedBenchmarkReport {
    CoreCheckedBenchmarkReport {
        benchmark_id: "storage".to_string(),
        core_check_gate: CoreCheckGateResult {
            core_check_ran: true,
            core_status: None,
            passed: true,
            failed_reasons: vec![],
            warnings: vec![],
            reason_codes: vec![],
        },
        dataset_selection: None,
        dataset_bundle: None,
        baseline_report: Some(PerformanceSummary::default()),
        external_report: None,
        calibration_report: None,
        model_comparison_report: None,
        risk_ai_interaction_report: None,
        storage_audit: BenchmarkStorageAudit {
            collection_bytes: total.saturating_sub(report),
            dataset_export_bytes: 0,
            prediction_bytes: 0,
            report_bytes: report,
            raw_archive_bytes: raw,
            canonical_bytes: canonical,
            budget_exceeded: false,
            largest_files: largest.iter().map(|value| value.to_string()).collect(),
            retention_actions: vec![],
            reason_codes: vec![],
        },
        usefulness_gate_result: ModelUsefulnessGateResult {
            passed: true,
            failed_gates: vec![],
            warnings: vec![],
            reason_codes: vec![],
        },
        final_status: CoreCheckedBenchmarkStatus::BaselineOnlyEvaluated,
        next_recommendation: CoreCheckedBenchmarkRecommendation::HoldCurrentScope,
        blockers: vec![],
        warnings: vec![],
        reason_codes: vec![ReasonCode::OfficialAiBenchmarkRan],
    }
}

#[test]
fn added_bytes_are_counted() {
    let previous = core_report(100, 10, 20, 5, &[]);
    let current = core_report(180, 20, 40, 10, &[]);
    let delta = build_official_storage_delta(Some(&previous), Some(&current), None, 1024);

    assert_eq!(delta.added_bytes, 80);
    assert_eq!(delta.added_raw_bytes, 10);
    assert_eq!(delta.added_canonical_bytes, 20);
    assert_eq!(delta.added_report_bytes, 5);
}

#[test]
fn budget_exceeded_is_reason_coded() {
    let current = core_report(2048, 100, 200, 50, &[]);
    let delta = build_official_storage_delta(None, Some(&current), None, 1024);

    assert!(delta.budget_exceeded);
    assert!(delta.reason_codes.contains(&ReasonCode::BudgetExceeded));
}

#[test]
fn largest_artifacts_are_sorted_deterministically() {
    let current = core_report(200, 10, 20, 30, &["z:1", "a:2", "m:3"]);
    let delta = build_official_storage_delta(None, Some(&current), None, 4096);

    assert_eq!(
        delta.largest_new_artifacts,
        vec!["a:2".to_string(), "m:3".to_string(), "z:1".to_string()]
    );
}

#[test]
fn compaction_recommendation_is_emitted_near_budget() {
    let current = core_report(950, 100, 200, 50, &[]);
    let delta = build_official_storage_delta(None, Some(&current), None, 1000);

    assert!(
        delta
            .compaction_recommendation
            .contains("Approaching storage budget")
    );
}

#[test]
fn retention_never_deletes_active_canonical_evidence_in_test() {
    let canonical = common::output_dir("storage-delta-retention").join("canonical.csv");
    fs::write(&canonical, "a,b\n1,2\n").expect("write canonical");
    let collection = OfficialCollectionReport {
        plan_id: "storage".to_string(),
        entry_reports: vec![],
        storage_budget_report: StorageBudgetReport {
            total_bytes: 10,
            canonical_bytes: 10,
            ..StorageBudgetReport::default()
        },
        ready_entries_count: 0,
        skipped_entries_count: 0,
        failed_entries_count: 0,
        official_api_collected_count: 0,
        reason_codes: vec![],
    };

    let _: OfficialStorageDelta = build_official_storage_delta(None, None, Some(&collection), 1000);
    assert!(canonical.exists());
}
