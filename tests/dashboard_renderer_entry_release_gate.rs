mod support;

use soma_zero::{
    DashboardRendererEntryReleaseGate, DashboardRendererEntryReleaseGateStatus,
    RealWorkspaceTimeoutAttributionConfig, Sprint93TimeoutAttributionRunner,
};
use support::{shared_fixture_harness as harness, sprint69_support as sprint};

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_dashboard_renderer_entry_release_gate.toml", name)
}

#[test]
fn dashboard_entry_release_gate_matches_expected_fixture() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_dashboard_renderer_entry_release_gate(&config("dashboard-entry-release-gate"))
        .expect("report");
    let mut expected = harness::load_json_fixture::<DashboardRendererEntryReleaseGate>(
        sprint::example_path("sprint93_data/dashboard_entry_release_gate_expected.json"),
    );
    expected.gate_id = report.gate_id.clone();
    assert_eq!(report, expected);
    assert_eq!(
        report.gate_status,
        DashboardRendererEntryReleaseGateStatus::DashboardRendererEntryReleased
    );
    assert!(report.entry_released);
}
