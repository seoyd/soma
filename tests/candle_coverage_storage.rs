use soma_zero::{CandleCoverageArtifactSize, build_candle_coverage_storage_report};

#[test]
fn candle_coverage_storage_counts_bytes_and_sorts_largest_artifacts() {
    let report = build_candle_coverage_storage_report(
        100,
        200,
        50,
        25,
        300,
        vec![
            CandleCoverageArtifactSize {
                path: "b".to_string(),
                bytes: 100,
            },
            CandleCoverageArtifactSize {
                path: "a".to_string(),
                bytes: 200,
            },
        ],
    );
    assert_eq!(report.total_bytes, 375);
    assert!(report.budget_exceeded);
    assert_eq!(report.largest_artifacts[0].path, "a");
    assert!(report.compaction_recommendation.contains("budget exceeded"));
}
