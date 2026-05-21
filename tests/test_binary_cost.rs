#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{
    RepeatedCommandTiming, TestBinaryCostKind, TestBinaryRecommendedAction, TimingRunCondition,
    build_repeated_workspace_timing_report, build_test_binary_cost_report,
};

#[test]
fn test_binary_cost_report_computes_primary_bottleneck() {
    let timing = build_repeated_workspace_timing_report(
        "timing",
        vec![
            RepeatedCommandTiming {
                command: "cargo test --workspace --quiet".to_string(),
                condition: TimingRunCondition::Warm,
                run_index: 0,
                wall_time_ms: Some(420000),
                exit_success: Some(true),
                stdout_summary: None,
                stderr_summary: None,
                reason_codes: Vec::new(),
            },
            RepeatedCommandTiming {
                command: "representative CLI smoke".to_string(),
                condition: TimingRunCondition::Warm,
                run_index: 0,
                wall_time_ms: Some(15000),
                exit_success: Some(true),
                stdout_summary: None,
                stderr_summary: None,
                reason_codes: Vec::new(),
            },
            RepeatedCommandTiming {
                command: "focused Sprint 77 tests".to_string(),
                condition: TimingRunCondition::Warm,
                run_index: 0,
                wall_time_ms: Some(82000),
                exit_success: Some(true),
                stdout_summary: None,
                stderr_summary: None,
                reason_codes: Vec::new(),
            },
        ],
        None,
        None,
    );
    let report = build_test_binary_cost_report("binary-cost", &timing);
    assert_eq!(
        report.primary_bottleneck.as_deref(),
        Some("cargo test --workspace --quiet")
    );
    assert!(
        report
            .records
            .iter()
            .any(|record| record.cost_kind == TestBinaryCostKind::RunHeavy)
    );
    assert!(
        report.records.iter().any(
            |record| record.recommended_action == TestBinaryRecommendedAction::MoveToSprintTier
        )
    );
}

#[test]
fn test_binary_cost_sample_covers_fixture_and_artifact_heavy_records() {
    let bundle = support::run_sprint77_bundle("soma_test_binary_cost.toml", "test-binary-cost");
    let records = &bundle.test_binary_cost_report.records;
    assert!(
        records
            .iter()
            .any(|record| record.cost_kind == TestBinaryCostKind::CompileHeavy)
    );
    assert!(
        records
            .iter()
            .any(|record| record.cost_kind == TestBinaryCostKind::RunHeavy)
    );
    assert!(
        records
            .iter()
            .any(|record| record.cost_kind == TestBinaryCostKind::FixtureHeavy)
    );
    assert!(
        records
            .iter()
            .any(|record| record.cost_kind == TestBinaryCostKind::CliSmokeHeavy)
    );
    assert!(
        records
            .iter()
            .any(|record| record.cost_kind == TestBinaryCostKind::ArtifactHeavy)
    );
}
