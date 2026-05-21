use soma_zero::{ArtifactSize, CorePerformanceBudget, ReasonCode, measure_performance_budget};

#[test]
fn budget_report_counts_rows_and_bytes_deterministically() {
    let budget = CorePerformanceBudget::default();
    let artifacts = vec![
        ArtifactSize {
            path: "b/report.txt".to_string(),
            bytes: 10,
        },
        ArtifactSize {
            path: "a/report.txt".to_string(),
            bytes: 20,
        },
    ];
    let left = measure_performance_budget(&budget, 10, 10, 10, 10, 30, 1, 10, &artifacts);
    let right = measure_performance_budget(&budget, 10, 10, 10, 10, 30, 1, 10, &artifacts);

    assert_eq!(left, right);
}

#[test]
fn budget_exceeded_is_reason_coded_and_sorted() {
    let budget = CorePerformanceBudget {
        max_artifact_bytes: 10,
        ..CorePerformanceBudget::default()
    };
    let report = measure_performance_budget(
        &budget,
        10,
        10,
        10,
        10,
        30,
        1,
        10,
        &[
            ArtifactSize {
                path: "b/report.txt".to_string(),
                bytes: 10,
            },
            ArtifactSize {
                path: "a/report.txt".to_string(),
                bytes: 20,
            },
        ],
    );

    assert!(report.budget_exceeded);
    assert!(report.reason_codes.contains(&ReasonCode::BudgetExceeded));
    assert!(
        report
            .reason_codes
            .contains(&ReasonCode::CompactionRecommended)
    );
    assert_eq!(report.largest_artifacts[0], "a/report.txt:20".to_string());
}
