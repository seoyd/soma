use soma_zero::{
    EvidenceSourceKind, LaneStorageBudgetReport, ReasonCode, build_provider_reality_storage_report,
    default_lane_storage_budget,
};

#[test]
fn lane_storage_budget_counts_bytes_and_near_budget_warning() {
    let budget =
        default_lane_storage_budget(EvidenceSourceKind::OfficialApiCollected, 500, 10, 60_000);
    assert!(budget.estimated_bytes > 0);

    let report = build_provider_reality_storage_report(
        vec![LaneStorageBudgetReport {
            lane_id: "lane-a".to_string(),
            estimated_bytes: 55_000,
            actual_bytes: Some(55_000),
            budget_ok: true,
            largest_artifacts: vec!["dataset.csv".to_string()],
            reason_codes: vec![ReasonCode::LaneStorageBudgetBuilt],
        }],
        60_000,
    );
    assert!(!report.budget_exceeded);
    assert!(
        report
            .compaction_recommendation
            .contains("Near budget limit")
    );
}
