mod support;

use soma_zero::{DashboardRendererCompileImpactStatus, Sprint94DashboardRendererRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn compile_impact_is_sample_backed_and_deterministic() {
    let first = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_compile_impact(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_compile_impact.toml",
            "dashboard-renderer-compile-impact-a",
        ))
        .expect("first");
    let second = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_compile_impact(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_compile_impact.toml",
            "dashboard-renderer-compile-impact-b",
        ))
        .expect("second");
    assert_eq!(first, second);
    assert_eq!(
        first.impact_status,
        DashboardRendererCompileImpactStatus::CompileImpactSampleBacked
    );
    assert_eq!(first.target_count_before, Some(7));
    assert_eq!(first.target_count_after, Some(3));
    assert_eq!(first.dashboard_family_delta, Some(-4));
}
