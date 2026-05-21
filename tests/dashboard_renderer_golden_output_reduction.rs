mod support;

use soma_zero::{DashboardRendererGoldenOutputStatus, Sprint94DashboardRendererRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn golden_output_reduction_stays_preserved() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_dashboard_renderer_golden_output_reduction(&sprint::sprint94_config_from_example(
            "soma_dashboard_renderer_golden_output_reduction.toml",
            "dashboard-renderer-golden",
        ))
        .expect("report");
    assert!(report.html_golden_checks_preserved);
    assert!(report.json_golden_checks_preserved);
    assert!(report.txt_golden_checks_preserved);
    assert!(report.duplicate_golden_checks_reduced);
    assert!(report.render_failure_still_fails);
    assert!(report.no_hidden_bless_update);
    assert_eq!(
        report.golden_status,
        DashboardRendererGoldenOutputStatus::GoldenOutputReduced
    );
}
