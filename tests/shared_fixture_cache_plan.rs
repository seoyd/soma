use soma_zero::{
    SharedFixtureCachePlanStatus, SharedFixtureCachePolicy, build_fixture_setup_cost_report,
    build_shared_fixture_cache_plan,
};

#[test]
fn shared_fixture_cache_plan_is_local_and_deterministic() {
    let config = soma_zero::RepeatedWorkspaceTimingConfig::default();
    let report = build_fixture_setup_cost_report("fixture-cost", &config);
    let plan = build_shared_fixture_cache_plan("fixture-cache", &report);
    assert!(matches!(
        plan.cache_policy,
        SharedFixtureCachePolicy::LocalInMemoryPerTestProcess
            | SharedFixtureCachePolicy::LocalOnDiskFingerprintCache
    ));
    assert!(
        plan.invalidation_rules
            .iter()
            .any(|rule| rule.contains("path"))
    );
    assert!(
        plan.invalidation_rules
            .iter()
            .any(|rule| rule.contains("content hash"))
    );
    assert_eq!(
        plan.plan_status,
        SharedFixtureCachePlanStatus::CachePlanReady
    );
    let json = serde_json::to_string(&plan).expect("serialize cache plan");
    assert!(!json.contains("SECRET"));
    assert_eq!(
        plan,
        build_shared_fixture_cache_plan("fixture-cache", &report)
    );
}
