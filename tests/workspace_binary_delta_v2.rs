mod support;

use soma_zero::{Sprint86ResidualGateRecoveryRunner, WorkspaceBinaryDeltaReportV2Status};
use support::sprint69_support as sprint;

#[test]
fn workspace_binary_delta_v2_reports_sample_backed_reduction() {
    let config = sprint::sprint86_config_from_example(
        "soma_workspace_binary_delta_v2.toml",
        "workspace-binary-delta-v2-test",
    );
    let report = Sprint86ResidualGateRecoveryRunner::default()
        .run_workspace_binary_delta_v2(&config)
        .expect("delta");
    assert_eq!(
        report.delta_status,
        WorkspaceBinaryDeltaReportV2Status::BinarySurfaceReducedWithWarnings
    );
    assert_eq!(report.binary_count_before, Some(20));
    assert_eq!(report.binary_count_after, Some(8));
    assert!(report.sample_backed);
    assert!(!report.measured);
}
