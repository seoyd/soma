use soma_zero::{
    TestSupportRefactorAction, TestSupportRefactorPlanStatus, build_fixture_setup_cost_report,
    build_test_support_refactor_plan,
};

#[test]
fn test_support_refactor_plan_identifies_centralization_targets() {
    let config = soma_zero::RepeatedWorkspaceTimingConfig::default();
    let report = build_fixture_setup_cost_report("fixture-cost", &config);
    let plan = build_test_support_refactor_plan("support-refactor", &report);
    assert!(
        plan.actions
            .contains(&TestSupportRefactorAction::CentralizeFixtureLoaders)
    );
    assert!(
        plan.actions
            .contains(&TestSupportRefactorAction::CentralizeTomlConfigBuilders)
    );
    assert!(
        plan.actions
            .contains(&TestSupportRefactorAction::CentralizeOutputDirSetup)
    );
    assert!(
        plan.actions
            .contains(&TestSupportRefactorAction::CentralizeCliSmokeHarness)
    );
    assert!(
        plan.actions
            .contains(&TestSupportRefactorAction::SplitHeavyIntegrationTest)
    );
    assert_eq!(
        plan.plan_status,
        TestSupportRefactorPlanStatus::NeedManualReview
    );
    assert_eq!(
        plan,
        build_test_support_refactor_plan("support-refactor", &report)
    );
}
