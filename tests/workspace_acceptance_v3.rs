use soma_zero::{
    CliSmokeCostReductionStatus, FixtureSetupDedupPlanStatus, RepeatedWorkspaceTimingStatus,
    TestTierKind, WorkspaceAcceptanceV3Status, build_test_tier_runner_report,
    build_workspace_acceptance_v3_report,
};

#[test]
fn workspace_acceptance_v3_requires_full_workspace_and_safety() {
    let report = build_workspace_acceptance_v3_report(
        "acceptance",
        "1.95.0",
        RepeatedWorkspaceTimingStatus::SampleBacked,
        FixtureSetupDedupPlanStatus::DedupPlanReady,
        CliSmokeCostReductionStatus::SmokeCostReduced,
        &[
            build_test_tier_runner_report(
                "quick",
                TestTierKind::Quick,
                vec!["check".to_string()],
                1,
                0,
                0,
                Some(1),
            ),
            build_test_tier_runner_report(
                "sprint",
                TestTierKind::Sprint,
                vec!["sprint".to_string()],
                1,
                0,
                0,
                Some(1),
            ),
            build_test_tier_runner_report(
                "full",
                TestTierKind::Full,
                vec!["cargo test --workspace --quiet".to_string()],
                1,
                0,
                0,
                Some(1),
            ),
            build_test_tier_runner_report(
                "audit",
                TestTierKind::Audit,
                vec!["audit".to_string()],
                1,
                0,
                0,
                Some(1),
            ),
        ],
    );
    assert_eq!(
        report.acceptance_status,
        WorkspaceAcceptanceV3Status::FullWorkspaceAccepted
    );
    assert!(report.safety_coverage_preserved);
    assert!(report.full_workspace_test_passed);
}

#[test]
fn workspace_acceptance_v3_fails_without_safety_or_full_gate() {
    let focused = build_workspace_acceptance_v3_report(
        "focused",
        "1.95.0",
        RepeatedWorkspaceTimingStatus::SampleBacked,
        FixtureSetupDedupPlanStatus::DedupPlanReady,
        CliSmokeCostReductionStatus::SmokeCostReduced,
        &[
            build_test_tier_runner_report(
                "quick",
                TestTierKind::Quick,
                vec!["check".to_string()],
                1,
                0,
                0,
                Some(1),
            ),
            build_test_tier_runner_report(
                "sprint",
                TestTierKind::Sprint,
                vec!["sprint".to_string()],
                1,
                0,
                0,
                Some(1),
            ),
        ],
    );
    assert_eq!(
        focused.acceptance_status,
        WorkspaceAcceptanceV3Status::FocusedOnly
    );

    let failed = build_workspace_acceptance_v3_report(
        "failed",
        "1.95.0",
        RepeatedWorkspaceTimingStatus::SampleBacked,
        FixtureSetupDedupPlanStatus::DedupPlanReady,
        CliSmokeCostReductionStatus::MissingRequiredSmoke,
        &[build_test_tier_runner_report(
            "full",
            TestTierKind::Full,
            vec!["cargo test --workspace --quiet".to_string()],
            1,
            0,
            0,
            Some(1),
        )],
    );
    assert_eq!(
        failed.acceptance_status,
        WorkspaceAcceptanceV3Status::Failed
    );
    assert!(!failed.safety_coverage_preserved);
}
