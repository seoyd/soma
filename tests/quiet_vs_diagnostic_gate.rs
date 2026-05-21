mod support;

use soma_zero::{
    QuietVsDiagnosticGateComparisonStatus, RealWorkspaceTimeoutAttributionConfig,
    Sprint93TimeoutAttributionRunner,
};
use support::sprint69_support as sprint;

fn config(name: &str) -> RealWorkspaceTimeoutAttributionConfig {
    sprint::sprint93_config_from_example("soma_quiet_vs_diagnostic_gate.toml", name)
}

#[test]
fn quiet_vs_diagnostic_gate_reports_improved_attribution() {
    let report = Sprint93TimeoutAttributionRunner::default()
        .run_quiet_vs_diagnostic_gate(&config("quiet-vs-diagnostic"))
        .expect("report");
    assert_eq!(
        report.comparison_status,
        QuietVsDiagnosticGateComparisonStatus::DiagnosticImprovedAttribution
    );
    assert!(report.output_visibility_improved);
    assert!(report.attribution_improved);
}
