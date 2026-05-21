use soma_zero::{
    FixtureDedupAction, build_fixture_setup_cost_report, build_fixture_setup_dedup_plan,
};

#[test]
fn fixture_setup_dedup_plan_covers_expected_actions() {
    let config = soma_zero::RepeatedWorkspaceTimingConfig::default();
    let report = build_fixture_setup_cost_report("fixture-cost", &config);
    let plan = build_fixture_setup_dedup_plan("fixture-plan", &report);
    assert!(plan.actions.contains(&FixtureDedupAction::ShareJsonFixture));
    assert!(plan.actions.contains(&FixtureDedupAction::ShareTomlFixture));
    assert!(
        plan.actions
            .contains(&FixtureDedupAction::PrecomputeFixtureIndex)
    );
    assert!(
        plan.actions
            .contains(&FixtureDedupAction::ReuseOutputDirTemplate)
    );
    assert_eq!(
        plan,
        build_fixture_setup_dedup_plan("fixture-plan", &report)
    );
}
