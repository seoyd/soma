use soma_zero::{
    TestRuntimeBudgetConfig, TestRuntimeBudgetStatus, TestTierKind,
    build_test_runtime_budget_report, build_test_tier_runner_report,
};

#[test]
fn runtime_budget_reports_within_over_and_missing_timing() {
    let config = TestRuntimeBudgetConfig {
        budget_id: "runtime-budget".to_string(),
        quick_budget_ms: 10_000,
        sprint_budget_ms: 20_000,
        full_budget_ms: 30_000,
        smoke_budget_ms: 10_000,
        audit_budget_ms: 10_000,
        reason_codes: Vec::new(),
    };
    let within = build_test_runtime_budget_report(
        &config,
        &[
            build_test_tier_runner_report(
                "quick",
                TestTierKind::Quick,
                vec!["check".to_string()],
                1,
                0,
                0,
                Some(1_000),
            ),
            build_test_tier_runner_report(
                "sprint",
                TestTierKind::Sprint,
                vec!["sprint".to_string()],
                1,
                0,
                0,
                Some(2_000),
            ),
            build_test_tier_runner_report(
                "full",
                TestTierKind::Full,
                vec!["full".to_string()],
                1,
                0,
                0,
                Some(3_000),
            ),
            build_test_tier_runner_report(
                "smoke",
                TestTierKind::Smoke,
                vec!["smoke".to_string()],
                1,
                0,
                0,
                Some(500),
            ),
            build_test_tier_runner_report(
                "audit",
                TestTierKind::Audit,
                vec!["audit".to_string()],
                1,
                0,
                0,
                Some(750),
            ),
        ],
    );
    assert_eq!(within.budget_status, TestRuntimeBudgetStatus::WithinBudget);

    let over = build_test_runtime_budget_report(
        &config,
        &[build_test_tier_runner_report(
            "full",
            TestTierKind::Full,
            vec!["full".to_string()],
            1,
            0,
            0,
            Some(35_000),
        )],
    );
    assert_eq!(over.budget_status, TestRuntimeBudgetStatus::OverBudget);
    assert!(over.over_budget_tiers.contains(&"full".to_string()));

    let missing = build_test_runtime_budget_report(&config, &[]);
    assert_eq!(
        missing.budget_status,
        TestRuntimeBudgetStatus::MissingTiming
    );
    assert_eq!(
        within,
        build_test_runtime_budget_report(
            &config,
            &[
                build_test_tier_runner_report(
                    "quick",
                    TestTierKind::Quick,
                    vec!["check".to_string()],
                    1,
                    0,
                    0,
                    Some(1_000)
                ),
                build_test_tier_runner_report(
                    "sprint",
                    TestTierKind::Sprint,
                    vec!["sprint".to_string()],
                    1,
                    0,
                    0,
                    Some(2_000)
                ),
                build_test_tier_runner_report(
                    "full",
                    TestTierKind::Full,
                    vec!["full".to_string()],
                    1,
                    0,
                    0,
                    Some(3_000)
                ),
                build_test_tier_runner_report(
                    "smoke",
                    TestTierKind::Smoke,
                    vec!["smoke".to_string()],
                    1,
                    0,
                    0,
                    Some(500)
                ),
                build_test_tier_runner_report(
                    "audit",
                    TestTierKind::Audit,
                    vec!["audit".to_string()],
                    1,
                    0,
                    0,
                    Some(750)
                ),
            ]
        )
    );
}
