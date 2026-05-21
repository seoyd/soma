mod support;

use soma_zero::{MeasuredTargetDeltaV5Status, Sprint89CandleRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn measured_target_delta_v5_keeps_sample_backed_state_explicit() {
    let config = sprint::sprint89_config_from_example(
        "soma_measured_target_delta_v5.toml",
        "measured-target-delta-v5",
    );
    let report = Sprint89CandleRecoveryRunner::default()
        .run_measured_target_delta_v5(&config)
        .expect("report");
    assert_eq!(
        report.delta_status,
        MeasuredTargetDeltaV5Status::SampleBackedOnly
    );
    assert_eq!(report.target_count_before, Some(2));
    assert_eq!(report.target_count_after, Some(1));
    assert_eq!(report.candle_family_delta, Some(1));
    assert!(!report.measured);
    assert!(report.sample_backed);
}
