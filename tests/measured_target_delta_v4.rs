mod support;

use soma_zero::{MeasuredTargetDeltaReportV4Status, Sprint88SevenBlockerRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn measured_target_delta_v4_stays_sample_backed_without_real_measurement() {
    let config = sprint::sprint88_config_from_example(
        "soma_measured_target_delta_v4.toml",
        "measured-delta",
    );
    let first = Sprint88SevenBlockerRecoveryRunner::default()
        .run_measured_target_delta_v4(&config)
        .expect("first");
    let second = Sprint88SevenBlockerRecoveryRunner::default()
        .run_measured_target_delta_v4(&config)
        .expect("second");
    assert!(!first.measured);
    assert!(first.sample_backed);
    assert_eq!(
        first.delta_status,
        MeasuredTargetDeltaReportV4Status::SampleBackedOnly
    );
    assert_eq!(first.family_target_deltas.len(), 7);
    assert_eq!(first, second);
}
