mod support;

use soma_zero::{MeasuredTargetDeltaStatusV10, Sprint94DashboardRendererRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn measured_target_delta_is_sample_backed() {
    let report = Sprint94DashboardRendererRecoveryRunner::default()
        .run_measured_target_delta_v10(&sprint::sprint94_config_from_example(
            "soma_measured_target_delta_v10.toml",
            "measured-target-delta-v10",
        ))
        .expect("report");
    assert_eq!(
        report.delta_status,
        MeasuredTargetDeltaStatusV10::SampleBackedOnly
    );
    assert!(!report.measured);
    assert!(report.sample_backed);
    assert_eq!(report.dashboard_family_delta, Some(-4));
}
