mod support;

use soma_zero::{
    DashboardRendererRealReductionAction, DashboardRendererRealReductionConfig,
    DashboardRendererRealReductionPlan, DashboardRendererRealReductionPlanStatus,
    DashboardRendererRealReductionStatus, Sprint94DashboardRendererRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str, output: &str) -> DashboardRendererRealReductionConfig {
    sprint::sprint94_config_from_example(name, output)
}

#[test]
fn config_defaults_and_remote_paths_stay_conservative() {
    let config = DashboardRendererRealReductionConfig::default();
    assert_eq!(config.target_family, "DashboardRenderer");
    assert!(config.preserve_assertions);
    assert!(config.preserve_safety_guards);
    assert!(config.preserve_static_read_only_checks);
    assert!(config.preserve_secret_redaction_checks);
    assert!(config.preserve_no_action_controls);
    assert!(config.preserve_no_browser_execution);
    assert!(config.preserve_determinism_checks);
    assert!(config.preserve_no_external_assets);
    let json = serde_json::to_string(&config).expect("json");
    assert!(!json.contains("runtime"));
    assert!(!json.contains("training"));
    assert!(!json.contains("broker"));
    assert!(!json.contains("order"));
    assert!(!json.contains("account"));

    let mut remote = config.clone();
    remote.output_root = "https://example.com/out".to_string();
    assert!(remote.validate().is_err());
}

#[test]
fn reduction_plan_matches_expected_fixture() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_real_reduction_plan(&config(
            "soma_dashboard_renderer_real_reduction_plan.toml",
            "dashboard-renderer-real-reduction-plan",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<DashboardRendererRealReductionPlan>(
        sprint::example_path("sprint94_data/dashboard_renderer_reduction_plan_expected.json"),
    );
    expected.plan_id = report.plan_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.plan_status,
        DashboardRendererRealReductionPlanStatus::DashboardReductionPlanReady
    );
    for action in [
        DashboardRendererRealReductionAction::VerifyGroupedSuiteCoverage,
        DashboardRendererRealReductionAction::MoveRemainingAssertions,
        DashboardRendererRealReductionAction::ApplySharedFixtureHarness,
        DashboardRendererRealReductionAction::ReduceDashboardStateFixtureDuplication,
        DashboardRendererRealReductionAction::ReduceHtmlGoldenDuplication,
        DashboardRendererRealReductionAction::ReduceJsonTxtGoldenDuplication,
        DashboardRendererRealReductionAction::ConsolidateSecretRedactionFixture,
    ] {
        assert!(report.actions.contains(&action));
    }
}

#[test]
fn reduction_report_threads_statuses() {
    let bundle = sprint::run_sprint94_bundle(
        "soma_sprint94_dashboard_renderer_recover.toml",
        "dashboard-renderer-real-reduction-bundle",
    );
    let report = bundle.dashboard_renderer_real_reduction_report;
    assert_eq!(
        report.reduction_status,
        DashboardRendererRealReductionStatus::DashboardRendererReducedWithWarnings
    );
    assert_eq!(
        report.reduction_plan_status,
        DashboardRendererRealReductionPlanStatus::DashboardReductionPlanReady
    );
}
