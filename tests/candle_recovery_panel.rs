mod support;

use soma_zero::{MeasuredTargetDeltaV5Status, Sprint89CandleRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn candle_recovery_panel_is_read_only_and_explicit() {
    let config = sprint::sprint89_config_from_example(
        "soma_control_tower_candle_recovery.toml",
        "candle-panel",
    );
    let report = Sprint89CandleRecoveryRunner::default()
        .run_control_tower_candle_recovery(&config)
        .expect("report");
    assert_eq!(report.primary_next_family, "ExternalPrediction");
    assert_eq!(
        report.measured_delta_status,
        MeasuredTargetDeltaV5Status::SampleBackedOnly
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|value| value.contains("no train button"))
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|value| value.contains("no runtime button"))
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|value| value.contains("no live button"))
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|value| value.contains("no order/account controls"))
    );
}
