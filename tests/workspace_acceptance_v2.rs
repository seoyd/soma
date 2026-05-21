use soma_zero::{
    TestTierKind, TestTierStatus, ToolchainVersionStatus, WorkspaceAcceptanceV2Status,
    build_test_tier_runner_report, build_workspace_acceptance_v2_report,
};

#[test]
fn workspace_acceptance_v2_reports_full_acceptance_only_with_full_workspace_pass() {
    let report = build_workspace_acceptance_v2_report(
        "workspace-full",
        ToolchainVersionStatus::LatestStablePinned,
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
                "smoke",
                TestTierKind::Smoke,
                vec!["smoke".to_string()],
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
        WorkspaceAcceptanceV2Status::FullWorkspaceAccepted
    );
    assert!(report.full_workspace_test_passed);
    assert_eq!(report.audit_status, TestTierStatus::TierPassed);
}

#[test]
fn workspace_acceptance_v2_distinguishes_focused_only_and_failed_states() {
    let focused_only = build_workspace_acceptance_v2_report(
        "workspace-focused",
        ToolchainVersionStatus::LatestStablePinned,
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
        focused_only.acceptance_status,
        WorkspaceAcceptanceV2Status::FocusedOnly
    );
    assert!(!focused_only.full_workspace_test_passed);

    let failed = build_workspace_acceptance_v2_report(
        "workspace-failed",
        ToolchainVersionStatus::LatestStablePinned,
        &[build_test_tier_runner_report(
            "full",
            TestTierKind::Full,
            vec!["cargo test --workspace --quiet".to_string()],
            0,
            1,
            0,
            Some(1),
        )],
    );
    assert_eq!(
        failed.acceptance_status,
        WorkspaceAcceptanceV2Status::Failed
    );
    assert_eq!(
        focused_only,
        build_workspace_acceptance_v2_report(
            "workspace-focused",
            ToolchainVersionStatus::LatestStablePinned,
            &[
                build_test_tier_runner_report(
                    "quick",
                    TestTierKind::Quick,
                    vec!["check".to_string()],
                    1,
                    0,
                    0,
                    Some(1)
                ),
                build_test_tier_runner_report(
                    "sprint",
                    TestTierKind::Sprint,
                    vec!["sprint".to_string()],
                    1,
                    0,
                    0,
                    Some(1)
                ),
            ],
        )
    );
}
