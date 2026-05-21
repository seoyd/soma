mod support;

use soma_zero::{MeasuredTargetDeltaV6Status, Sprint90ExternalPredictionRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn measured_target_delta_v6_keeps_sample_backed_state_explicit() {
    let config = sprint::sprint90_config_from_example(
        "soma_measured_target_delta_v6.toml",
        "measured-target-delta-v6",
    );
    let report = Sprint90ExternalPredictionRecoveryRunner::default()
        .run_measured_target_delta_v6(&config)
        .expect("report");
    assert_eq!(
        report.delta_status,
        MeasuredTargetDeltaV6Status::SampleBackedOnly
    );
    assert_eq!(report.target_count_before, Some(6));
    assert_eq!(report.target_count_after, Some(5));
    assert_eq!(report.external_family_delta, Some(1));
    assert!(!report.measured);
    assert!(report.sample_backed);
}
