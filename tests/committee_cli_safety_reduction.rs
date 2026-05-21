mod support;

use soma_zero::{
    CommitteeCliSafetyReductionAction, CommitteeCliSafetyReductionConfig,
    CommitteeCliSafetyReductionPlan, CommitteeCliSafetyReductionPlanStatus,
    CommitteeCliSafetyReductionStatus, Sprint95CommitteeCliSafetyRecoveryRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str, output: &str) -> CommitteeCliSafetyReductionConfig {
    sprint::sprint95_config_from_example(name, output)
}

#[test]
fn config_defaults_and_remote_paths_stay_conservative() {
    let config = CommitteeCliSafetyReductionConfig::default();
    assert_eq!(config.target_family, "CommitteeCliSafety");
    assert!(config.preserve_assertions);
    assert!(config.preserve_safety_guards);
    assert!(config.preserve_remote_path_rejection);
    assert!(config.preserve_help_text_checks);
    assert!(config.preserve_forbidden_command_checks);
    assert!(config.preserve_runtime_deferred_checks);
    assert!(config.preserve_persona_expansion_guards);
    assert!(config.preserve_order_account_guards);
    assert!(config.preserve_browser_execution_guards);
    assert!(config.require_isolation_decision);
    let json = serde_json::to_string(&config).expect("json");
    assert!(!json.contains("training"));
    assert!(!json.contains("live_trading"));
    assert!(!json.contains("runtime_llm"));
    assert!(!json.contains("https://"));

    let mut remote = config.clone();
    remote.output_root = "https://example.com/out".to_string();
    assert!(remote.validate().is_err());
}

#[test]
fn reduction_plan_matches_expected_fixture() {
    let report = Sprint95CommitteeCliSafetyRecoveryRunner::default()
        .run_committee_cli_safety_reduction_plan(&config(
            "soma_committee_cli_safety_reduction_plan.toml",
            "committee-cli-safety-reduction-plan",
        ))
        .expect("report");
    let mut expected = harness::load_json_fixture::<CommitteeCliSafetyReductionPlan>(
        sprint::example_path("sprint95_data/committee_cli_safety_reduction_plan_expected.json"),
    );
    expected.plan_id = report.plan_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.plan_status,
        CommitteeCliSafetyReductionPlanStatus::CommitteeCliSafetyPlanReady
    );
    for action in [
        CommitteeCliSafetyReductionAction::VerifyWorkspaceCliSafetySuiteCoverage,
        CommitteeCliSafetyReductionAction::VerifyWorkspaceSafetyGuardSuiteCoverage,
        CommitteeCliSafetyReductionAction::KeepIsolatedSentinel,
        CommitteeCliSafetyReductionAction::SplitHighRiskCliSafety,
        CommitteeCliSafetyReductionAction::ReduceHelpTextFixtureDuplication,
        CommitteeCliSafetyReductionAction::ReduceCommandSurfaceFixtureDuplication,
    ] {
        assert!(report.actions.contains(&action));
    }
}

#[test]
fn reduction_report_keeps_committee_cli_safety_explicitly_isolated() {
    let bundle = sprint::run_sprint95_bundle(
        "soma_sprint95_committee_cli_safety_recover.toml",
        "committee-cli-safety-bundle",
    );
    assert_eq!(
        bundle
            .committee_cli_safety_reduction_report
            .reduction_status,
        CommitteeCliSafetyReductionStatus::CommitteeCliSafetyKeptIsolated
    );
}
