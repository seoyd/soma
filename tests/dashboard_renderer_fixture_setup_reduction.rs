mod support;

use soma_zero::{
    DashboardRendererFixtureSetupReductionStatus, Sprint94DashboardRendererRecoveryRunner,
};
use support::sprint69_support as sprint;

#[test]
fn fixture_setup_reduction_is_reported_and_deterministic() {
    let first = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_fixture_setup_reduction(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_fixture_setup_reduction.toml",
            "dashboard-renderer-fixture-setup-a",
        ))
        .expect("first");
    let second = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_fixture_setup_reduction(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_fixture_setup_reduction.toml",
            "dashboard-renderer-fixture-setup-b",
        ))
        .expect("second");
    assert_eq!(first, second);
    assert_eq!(
        first.reduction_status,
        DashboardRendererFixtureSetupReductionStatus::FixtureSetupReduced
    );
    assert!(first.shared_fixture_harness_used);
    assert!(first.deterministic_output_preserved);
    assert_eq!(first.duplicate_output_dirs_removed, 4);
}
