#[path = "support/sprint69_support.rs"]
mod support;

use soma_zero::{
    DeveloperSpeedRunbook, DeveloperSpeedRunbookStatus, OptionalNextestPlanStatus,
    OptionalSccachePlanStatus, build_developer_speed_runbook, build_optional_nextest_plan,
    build_optional_sccache_plan, build_test_tier_config,
};

#[test]
fn nextest_and_sccache_plans_cover_available_and_unavailable_states() {
    let nextest_unavailable = build_optional_nextest_plan(false);
    assert_eq!(
        nextest_unavailable.plan_status,
        OptionalNextestPlanStatus::NextestRecommended
    );
    assert!(!nextest_unavailable.fallback_cargo_test_commands.is_empty());

    let nextest_available = build_optional_nextest_plan(true);
    assert_eq!(
        nextest_available.plan_status,
        OptionalNextestPlanStatus::NextestReady
    );

    let sccache_unavailable = build_optional_sccache_plan(false);
    assert_eq!(
        sccache_unavailable.plan_status,
        OptionalSccachePlanStatus::SccacheRecommended
    );
    assert!(sccache_unavailable.local_cache_only);
    let serialized = serde_json::to_string(&sccache_unavailable).expect("serialize sccache plan");
    assert!(!serialized.contains("SECRET"));
    assert!(!serialized.contains("TOKEN"));

    let sccache_available = build_optional_sccache_plan(true);
    assert_eq!(
        sccache_available.plan_status,
        OptionalSccachePlanStatus::SccacheReady
    );
}

#[test]
fn developer_speed_runbook_matches_expected_fixture() {
    let tier_config = build_test_tier_config("expected-speed-runbook");
    let runbook = build_developer_speed_runbook(
        "expected-speed-runbook",
        &tier_config,
        &build_optional_nextest_plan(false),
        &build_optional_sccache_plan(false),
    );
    let expected: DeveloperSpeedRunbook = support::read_json(support::example_path(
        "sprint76_data/expected_speed_runbook.json",
    ));
    assert_eq!(runbook, expected);
    assert_eq!(
        runbook.runbook_status,
        DeveloperSpeedRunbookStatus::RunbookReadyWithWarnings
    );
}
