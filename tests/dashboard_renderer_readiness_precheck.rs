mod support;

use std::fs;

use soma_zero::{
    DashboardRendererReadinessPrecheckStatus, KrxEvidenceWarningClosureConfig,
    Sprint92KrxWarningClosureRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> KrxEvidenceWarningClosureConfig {
    sprint::sprint92_config_from_example("soma_dashboard_renderer_readiness_precheck.toml", name)
}

#[test]
fn dashboard_precheck_matches_expected_fixture_and_is_deterministic() {
    let runner = Sprint92KrxWarningClosureRunner::default();
    let config = config("dashboard-precheck-default");
    let first = runner
        .run_dashboard_renderer_readiness_precheck(&config)
        .expect("first");
    let second = runner
        .run_dashboard_renderer_readiness_precheck(&config)
        .expect("second");
    let expected = harness::load_json_fixture(sprint::example_path(
        "sprint92_data/dashboard_renderer_precheck_expected.json",
    ));
    assert_eq!(first, expected);
    assert_eq!(first, second);
    assert_eq!(
        first.precheck_status,
        DashboardRendererReadinessPrecheckStatus::DashboardRendererPrecheckReady
    );
}

#[test]
fn dashboard_precheck_can_block_when_checks_are_missing() {
    let mut config = config("dashboard-precheck-blocked");
    let dir = harness::temp_output_dir_for_test("dashboard-precheck-blocked");
    let suite = dir.join("dashboard_renderer_suite.rs");
    let renderer = dir.join("dashboard_renderer.rs");
    let artifact = dir.join("artifact_rendering_suite.rs");
    fs::write(&suite, "#[test]\nfn weak() { assert!(true); }\n").expect("write suite");
    fs::write(
        &renderer,
        "pub fn render() -> &'static str { \"minimal\" }\n",
    )
    .expect("write renderer");
    fs::write(
        &artifact,
        "#[test]\nfn weak_artifact() { assert!(true); }\n",
    )
    .expect("write artifact");
    config
        .workspace_gate_paths
        .insert(0, suite.display().to_string());
    config
        .workspace_gate_paths
        .insert(0, renderer.display().to_string());
    config
        .workspace_gate_paths
        .insert(0, artifact.display().to_string());
    let report = Sprint92KrxWarningClosureRunner::default()
        .run_dashboard_renderer_readiness_precheck(&config)
        .expect("report");
    assert_eq!(
        report.precheck_status,
        DashboardRendererReadinessPrecheckStatus::DashboardRendererPrecheckBlocked
    );
}
